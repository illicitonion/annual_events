use annual_events::make_calendar;

fn main() {
    make_calendar(std::io::stdout()).unwrap();
}
