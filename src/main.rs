use daikoku::Dkk;

use daikoku::alias::DkkResult;
use daikoku::error::DkkError;

#[tokio::main]
async fn main() -> DkkResult<()> {
    eframe::run_native(
        "Dkk",
        eframe::NativeOptions::default(),
        Box::new(|_| Box::new(Dkk::new())),
    )
    .map_err(DkkError::Render)
}
