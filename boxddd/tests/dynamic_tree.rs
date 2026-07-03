use boxddd::{
    Aabb, BoxCastInput, DynamicTree, DynamicTreeCastControl, DynamicTreeFilter, Error,
    RayCastInput, Vec3,
};

fn aabb(lower: f32, upper: f32) -> Aabb {
    Aabb {
        lower_bound: Vec3::new(lower, lower, lower),
        upper_bound: Vec3::new(upper, upper, upper),
    }
}

fn x_sweep_box(lower_x: f32, upper_x: f32) -> Aabb {
    Aabb {
        lower_bound: Vec3::new(lower_x, -0.5, -0.5),
        upper_bound: Vec3::new(upper_x, 0.5, 0.5),
    }
}

#[test]
fn proxy_lifecycle_query_and_stale_ids_are_safe() -> boxddd::Result<()> {
    let mut tree = DynamicTree::new()?;
    assert_eq!(tree.proxy_count()?, 0);
    assert_eq!(tree.root_bounds()?, None);

    let proxy_id = tree.create_proxy(aabb(-1.0, 1.0), 42)?;
    assert!(proxy_id.generation() > 0);
    assert!(tree.contains_proxy(proxy_id));
    assert_eq!(tree.proxy_count()?, 1);
    assert_eq!(tree.proxy(proxy_id)?.user_data, 42);
    assert!(tree.byte_count()? > 0);
    assert!(tree.area_ratio()? >= 0.0);
    assert!(tree.height()? >= 0);
    assert!(tree.root_bounds()?.is_some());
    tree.validate()?;
    tree.validate_no_enlarged()?;

    let hits = tree.query(aabb(-0.5, 0.5), DynamicTreeFilter::default())?;
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].proxy_id, proxy_id);
    assert_eq!(hits[0].user_data, 42);
    assert_eq!(tree.proxy(hits[0].proxy_id)?.user_data, 42);

    tree.destroy_proxy(proxy_id)?;
    assert!(!tree.contains_proxy(proxy_id));
    assert_eq!(tree.destroy_proxy(proxy_id), Err(Error::InvalidArgument));
    assert!(
        tree.query(aabb(-0.5, 0.5), DynamicTreeFilter::default())?
            .is_empty()
    );

    let replacement = tree.create_proxy(aabb(-1.0, 1.0), 84)?;
    if replacement.index() == proxy_id.index() {
        assert_ne!(replacement.generation(), proxy_id.generation());
    }
    assert_eq!(tree.proxy(proxy_id), Err(Error::InvalidArgument));
    assert_eq!(tree.proxy(replacement)?.user_data, 84);
    Ok(())
}

#[test]
fn moving_enlarging_and_rebuilding_update_queries() -> boxddd::Result<()> {
    let mut tree = DynamicTree::new()?;
    let proxy_id = tree.create_proxy(aabb(-1.0, 1.0), 7)?;

    assert!(
        tree.query(aabb(4.0, 5.0), DynamicTreeFilter::default())?
            .is_empty()
    );

    tree.move_proxy(proxy_id, aabb(4.0, 5.0))?;
    tree.validate_no_enlarged()?;
    let moved_hits = tree.query(aabb(4.25, 4.75), DynamicTreeFilter::default())?;
    assert_eq!(moved_hits[0].proxy_id, proxy_id);

    assert_eq!(
        tree.enlarge_proxy(proxy_id, aabb(4.1, 4.9)),
        Err(Error::InvalidArgument)
    );
    tree.enlarge_proxy(proxy_id, aabb(3.0, 6.0))?;
    assert_eq!(tree.validate_no_enlarged(), Err(Error::InvalidArgument));

    let enlarged_hits = tree.query(aabb(3.1, 3.2), DynamicTreeFilter::default())?;
    assert_eq!(enlarged_hits[0].proxy_id, proxy_id);

    tree.rebuild(false)?;
    tree.validate_no_enlarged()?;
    Ok(())
}

#[test]
fn category_masks_and_require_all_bits_filter_proxies() -> boxddd::Result<()> {
    let mut tree = DynamicTree::new()?;
    let a = tree.create_proxy_with_category_bits(aabb(-1.0, 1.0), 0b0011, 1)?;
    let b = tree.create_proxy_with_category_bits(aabb(-1.0, 1.0), 0b0101, 2)?;

    let any_bit = tree.query(aabb(-0.5, 0.5), DynamicTreeFilter::new(0b0001))?;
    assert_eq!(any_bit.len(), 2);

    let mut stopped_after_first = Vec::new();
    tree.visit_query(aabb(-0.5, 0.5), DynamicTreeFilter::new(0b0001), |hit| {
        stopped_after_first.push(hit.proxy_id);
        false
    })?;
    assert_eq!(stopped_after_first.len(), 1);

    let require_all = DynamicTreeFilter::new(0b0011).require_all_bits(true);
    let all_bits = tree.query(aabb(-0.5, 0.5), require_all)?;
    assert_eq!(all_bits.len(), 1);
    assert_eq!(all_bits[0].proxy_id, a);

    tree.set_category_bits(b, 0b0011)?;
    assert_eq!(tree.category_bits(b)?, 0b0011);
    let all_bits = tree.query(aabb(-0.5, 0.5), require_all)?;
    assert_eq!(all_bits.len(), 2);
    assert!(all_bits.iter().any(|hit| hit.proxy_id == a));
    assert!(all_bits.iter().any(|hit| hit.proxy_id == b));
    Ok(())
}

#[test]
fn closest_ray_and_box_cast_callbacks_return_owned_ids() -> boxddd::Result<()> {
    let mut tree = DynamicTree::new()?;
    let near = tree.create_proxy(aabb(-1.0, 1.0), 10)?;
    let far = tree.create_proxy(aabb(5.0, 6.0), 20)?;

    let mut closest_seen = Vec::new();
    let closest = tree.query_closest(
        Vec3::ZERO,
        DynamicTreeFilter::default(),
        1_000_000.0,
        |hit| {
            closest_seen.push(hit.proxy_id);
            if hit.proxy_id == near { 0.0 } else { 100.0 }
        },
    )?;
    assert!(closest.stats.leaf_visits >= 1);
    assert_eq!(closest.min_distance_squared, 0.0);
    assert!(closest_seen.contains(&near));

    let mut ray_hits = Vec::new();
    let ray_stats = tree.ray_cast(
        RayCastInput::new(Vec3::new(-5.0, 0.0, 0.0), Vec3::new(20.0, 0.0, 0.0))?,
        DynamicTreeFilter::default(),
        |hit| {
            ray_hits.push(hit.proxy_id);
            DynamicTreeCastControl::Clip(0.4)
        },
    )?;
    assert!(ray_stats.leaf_visits >= 1);
    assert_eq!(ray_hits, vec![near]);
    assert!(!ray_hits.contains(&far));

    let mut box_hits = Vec::new();
    let box_stats = tree.box_cast(
        BoxCastInput::new(x_sweep_box(-5.0, -4.5), Vec3::new(20.0, 0.0, 0.0))?,
        DynamicTreeFilter::default(),
        |hit| {
            box_hits.push(hit.proxy_id);
            DynamicTreeCastControl::Clip(0.4)
        },
    )?;
    assert!(box_stats.leaf_visits >= 1);
    assert_eq!(box_hits, vec![near]);
    assert!(!box_hits.contains(&far));
    Ok(())
}

#[test]
fn dynamic_tree_callback_panics_are_reported() -> boxddd::Result<()> {
    let mut tree = DynamicTree::new()?;
    let proxy_id = tree.create_proxy(aabb(-1.0, 1.0), 1)?;

    let mut reentrant_error = None;
    tree.visit_query(aabb(-0.5, 0.5), DynamicTreeFilter::default(), |hit| {
        assert_eq!(hit.proxy_id, proxy_id);
        reentrant_error = Some(tree.proxy(hit.proxy_id).unwrap_err());
        true
    })?;
    assert_eq!(reentrant_error, Some(Error::InCallback));

    assert_eq!(
        tree.visit_query(aabb(-0.5, 0.5), DynamicTreeFilter::default(), |_| {
            panic!("query panic");
        }),
        Err(Error::CallbackPanicked)
    );
    assert_eq!(
        tree.query_closest(Vec3::ZERO, DynamicTreeFilter::default(), 100.0, |_| {
            panic!("closest panic");
        }),
        Err(Error::CallbackPanicked)
    );
    assert_eq!(
        tree.ray_cast(
            RayCastInput::new(Vec3::new(-5.0, 0.0, 0.0), Vec3::new(10.0, 0.0, 0.0))?,
            DynamicTreeFilter::default(),
            |_| panic!("ray panic"),
        ),
        Err(Error::CallbackPanicked)
    );
    assert_eq!(
        tree.box_cast(
            BoxCastInput::new(x_sweep_box(-5.0, -4.5), Vec3::new(10.0, 0.0, 0.0))?,
            DynamicTreeFilter::default(),
            |_| panic!("box panic"),
        ),
        Err(Error::CallbackPanicked)
    );
    Ok(())
}

#[test]
fn invalid_dynamic_tree_inputs_return_errors() -> boxddd::Result<()> {
    let mut tree = DynamicTree::new()?;
    let invalid_aabb = Aabb {
        lower_bound: Vec3::new(1.0, 1.0, 1.0),
        upper_bound: Vec3::new(-1.0, -1.0, -1.0),
    };
    assert_eq!(
        tree.create_proxy(invalid_aabb, 0),
        Err(Error::InvalidArgument)
    );

    let proxy_id = tree.create_proxy(aabb(-1.0, 1.0), 1)?;
    assert_eq!(
        tree.move_proxy(proxy_id, invalid_aabb),
        Err(Error::InvalidArgument)
    );
    assert_eq!(
        tree.enlarge_proxy(proxy_id, aabb(-0.5, 0.5)),
        Err(Error::InvalidArgument)
    );
    tree.destroy_proxy(proxy_id)?;
    assert_eq!(
        tree.move_proxy(proxy_id, aabb(2.0, 3.0)),
        Err(Error::InvalidArgument)
    );

    assert_eq!(
        RayCastInput::with_max_fraction(Vec3::ZERO, Vec3::X, -0.1),
        Err(Error::InvalidArgument)
    );
    assert_eq!(
        BoxCastInput::new(aabb(-1.0, 1.0), Vec3::new(f32::NAN, 0.0, 0.0)),
        Err(Error::InvalidArgument)
    );
    assert_eq!(
        tree.query(invalid_aabb, DynamicTreeFilter::default()),
        Err(Error::InvalidArgument)
    );
    assert_eq!(
        tree.query_closest(Vec3::ZERO, DynamicTreeFilter::default(), -1.0, |_| 0.0),
        Err(Error::InvalidArgument)
    );
    tree.create_proxy(aabb(-1.0, 1.0), 2)?;
    assert_eq!(
        tree.ray_cast(
            RayCastInput::new(Vec3::new(-5.0, 0.0, 0.0), Vec3::new(10.0, 0.0, 0.0))?,
            DynamicTreeFilter::default(),
            |_| DynamicTreeCastControl::Clip(f32::NAN),
        ),
        Err(Error::InvalidArgument)
    );
    Ok(())
}
