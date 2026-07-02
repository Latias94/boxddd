mod common;

use anyhow::{Context, Result};
use common::BodySnapshot;
use std::sync::mpsc;
use std::thread;

enum PhysicsCommand {
    Step { count: usize },
    Shutdown,
}

fn main() -> Result<()> {
    let (command_tx, command_rx) = mpsc::channel::<PhysicsCommand>();
    let (snapshot_tx, snapshot_rx) = mpsc::channel::<Vec<BodySnapshot>>();

    let worker = thread::spawn(move || -> Result<()> {
        let mut scene = common::falling_stack_scene().context("failed to create physics world")?;
        while let Ok(command) = command_rx.recv() {
            match command {
                PhysicsCommand::Step { count } => {
                    for _ in 0..count {
                        scene.step(1.0 / 60.0, 4)?;
                    }
                    snapshot_tx
                        .send(scene.snapshots()?)
                        .context("render side dropped the snapshot receiver")?;
                }
                PhysicsCommand::Shutdown => break,
            }
        }
        Ok(())
    });

    command_tx.send(PhysicsCommand::Step { count: 120 })?;
    let snapshots = snapshot_rx.recv()?;
    for snapshot in snapshots.iter().take(4) {
        println!(
            "{:<6} from physics thread: y={:.3}",
            snapshot.label, snapshot.position.y
        );
    }

    command_tx.send(PhysicsCommand::Shutdown)?;
    worker
        .join()
        .expect("physics thread panicked")
        .context("physics thread failed")?;

    Ok(())
}
