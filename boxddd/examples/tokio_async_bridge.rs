mod common;

use anyhow::{Context, Result};
use common::BodySnapshot;
use tokio::sync::{mpsc, oneshot};

enum PhysicsCommand {
    Step {
        count: usize,
        reply: oneshot::Sender<Vec<BodySnapshot>>,
    },
    Shutdown,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let (command_tx, mut command_rx) = mpsc::channel::<PhysicsCommand>(8);

    let worker = tokio::task::spawn_blocking(move || -> Result<()> {
        let mut scene = common::falling_stack_scene().context("failed to create physics world")?;
        while let Some(command) = command_rx.blocking_recv() {
            match command {
                PhysicsCommand::Step { count, reply } => {
                    for _ in 0..count {
                        scene.step(1.0 / 60.0, 4)?;
                    }
                    let _ = reply.send(scene.snapshots()?);
                }
                PhysicsCommand::Shutdown => break,
            }
        }
        Ok(())
    });

    let (reply_tx, reply_rx) = oneshot::channel();
    command_tx
        .send(PhysicsCommand::Step {
            count: 90,
            reply: reply_tx,
        })
        .await
        .context("physics task stopped before receiving work")?;

    let snapshots = reply_rx.await.context("physics task dropped reply")?;
    println!(
        "async bridge received {} body snapshots; first y={:.3}",
        snapshots.len(),
        snapshots
            .first()
            .map(|snapshot| snapshot.position.y)
            .unwrap_or_default()
    );

    command_tx.send(PhysicsCommand::Shutdown).await.ok();
    worker.await.context("physics task panicked")??;

    Ok(())
}
