macro_rules! m { ($($t:tt)*) => {}; }
m!(((([[{{}}]]))));
#[cfg(all(test, not(miri)))]
mod tests { use super::*; }
