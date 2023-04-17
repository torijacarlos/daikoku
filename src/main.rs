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

// @todo: figure out what to do with ui, I don't want to invest a lot on it for now, but I think
// the next steps require some sort of polish. take notes on each scene and act on that maybe
//
// @todo: export to encrypted file
// @todo: import it to database
