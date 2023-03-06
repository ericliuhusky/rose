mod config;
mod makefile;
mod link_app;

use makefile::create_makefile;
use link_app::create_link_app;

use config::ch;

fn main() {
    config::init();

    for ch in ch() {
        create_makefile(ch);
        create_link_app(ch);
    }
}
