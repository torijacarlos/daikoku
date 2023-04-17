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
//
// @todo: the past rabbit hole made me think, do I even need a database?
// I'll try to make it an option. or should i delete the entire db implementation?
// where and why would I need it? The file could get a bit big, but even if it gets to 1GB, I could
// start splitting it into separate files
// I struggled so much with the threading that it pains me to do so
