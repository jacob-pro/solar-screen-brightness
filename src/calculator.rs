//! Algorithm for computing the brightness percentage for a given time

use sunrise_sunset_calculator::SunriseSunsetResult;

#[derive(Debug)]
pub struct BrightnessResult {
    /// Unix time that the brightness should be re-calculated
    pub expiry_time: Option<i64>,
    /// Brightness percentage for the current time
    pub brightness: u32,
}

fn sine_curve(
    time_now: i64,
    transition: u32,
    event_time: i64,
    decreasing: bool,
    low_brightness: u32,
    high_brightness: u32,
) -> BrightnessResult {
    // We need to transform the sine function
    // Scale the height to the difference between min and max brightness
    let y_multiplier = (high_brightness - low_brightness) as f64 / 2.0;
    // Shift upwards to the midpoint brightness
    let y_offset = y_multiplier + low_brightness as f64;
    // Scale half a cycle to be equal to the transition time
    let mut x_multiplier = std::f64::consts::PI / transition as f64;
    // Flip the curve to make it a decreasing function
    if decreasing {
        x_multiplier = -x_multiplier;
    }
    // Shift rightwards to centre on the sunrise/sunset event
    let x_offset = (time_now - event_time) as f64;
    // Compute brightness
    let brightness = (y_multiplier * (x_multiplier * x_offset).sin()) + y_offset;
    let brightness = brightness.round() as u32; // round to nearest integer brightness

    // Work out the expiry time; when the brightness will change to the next integer value
    let mut next_update_brightness = if decreasing {
        brightness - 1
    } else {
        brightness + 1
    };
    if next_update_brightness > high_brightness {
        next_update_brightness = high_brightness;
    } else if next_update_brightness < low_brightness {
        next_update_brightness = low_brightness;
    }
    let expiry_time = if time_now == event_time {
        time_now + 1 // Don't get stuck into an infinite loop when exactly on the boundary
    } else {
        // Inverse of the sine function at next_update_brightness
        let asin = ((next_update_brightness as f64 - y_offset) / y_multiplier).asin();
        let expiry_offset = (asin / x_multiplier).round();
        expiry_offset as i64 + event_time
    };
    BrightnessResult {
        expiry_time: Some(expiry_time),
        brightness,
    }
}

pub fn calculate_brightness(
    brightness_day: u32,
    brightness_night: u32,
    transition_mins: u32,
    sun: &SunriseSunsetResult,
    time_now: i64,
) -> BrightnessResult {
    // Special case where there is no difference in brightness
    if brightness_night == brightness_day {
        return BrightnessResult {
            expiry_time: None,
            brightness: brightness_day,
        };
    }

    let low = brightness_day.min(brightness_night);
    let high = brightness_day.max(brightness_night);
    let transition_secs = transition_mins * 60; //time for transition from low to high
    let half_transition_secs = (transition_secs / 2) as i64;

    let (time_a, time_b) = if sun.visible {
        // Daytime
        (
            sun.rise + half_transition_secs, // When the sun rose this morning  + transition
            sun.set - half_transition_secs,
        ) // Whe the sun sets this evening - transition
    } else {
        // Nighttime
        (
            sun.set + half_transition_secs, // When the sun set at the start of night + transition
            sun.rise - half_transition_secs,
        ) // When the sun will rise again - transition
    };

    // If nighttime brightness is greater than day (weird!) then we need to flip around.
    let backwards = brightness_night > brightness_day;

    if time_now < time_a {
        let event = if sun.visible { sun.rise } else { sun.set };
        sine_curve(
            time_now,
            transition_secs,
            event,
            !(sun.visible ^ backwards),
            low,
            high,
        )
    } else if time_now >= time_b {
        // Must be greater or equal to or it would get stuck in a loop
        let event = if sun.visible { sun.set } else { sun.rise };
        sine_curve(
            time_now,
            transition_secs,
            event,
            sun.visible ^ backwards,
            low,
            high,
        )
    } else {
        // Time is >=A and <B, therefore the brightness next change is at B
        BrightnessResult {
            expiry_time: Some(time_b),
            brightness: if sun.visible {
                brightness_day
            } else {
                brightness_night
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};

    #[test]
    fn test_sunset_sine_curve() {
        let low = 40;
        let high = 80;
        let t_secs = 60 * 60; //60 minutes
        let set = Utc.ymd(2018, 12, 2).and_hms(16, 0, 0).timestamp(); // Fictional
        let midpoint = (low as f64 + ((high - low) as f64 / 2.0)).round() as u32;

        // At start of the transition it should equal the day brightness
        let transition_start = Utc.ymd(2018, 12, 2).and_hms(15, 30, 0).timestamp();
        let r = sine_curve(transition_start, t_secs, set, true, low, high);
        assert_eq!(high, r.brightness); //80

        //Test part way between transition. It should be less than the daytime brightness. But greater than the midpoint because it is not yet sunset
        let before_sunset = Utc.ymd(2018, 12, 2).and_hms(15, 45, 0).timestamp();
        let r = sine_curve(before_sunset, t_secs, set, true, low, high);
        assert!(r.brightness < high && r.brightness > midpoint); //~74

        //At sunset it should be half way between the day and night brightness
        let r = sine_curve(set, t_secs, set, true, low, high);
        assert_eq!(midpoint, r.brightness); //60

        //At end of the transition it should equal the night brightness
        let transition_end = Utc.ymd(2018, 12, 2).and_hms(16, 30, 0).timestamp();
        let r = sine_curve(transition_end, t_secs, set, true, low, high);
        assert_eq!(r.brightness, low); // 40
    }

    #[test]
    fn test_sunrise_sine_curve() {
        let low = 35;
        let high = 76;
        let t_secs = 40 * 60; //40 minutes
        let rise = Utc.ymd(2018, 12, 2).and_hms(8, 0, 0).timestamp(); // Fictional
        let midpoint = (low as f64 + ((high - low) as f64 / 2.0)).round() as u32;

        //At start of the transition it should equal the night brightness
        let start_of_transition = Utc.ymd(2018, 12, 2).and_hms(7, 40, 0).timestamp();
        let r = sine_curve(start_of_transition, t_secs, rise, false, low, high);
        assert_eq!(low, r.brightness); // 35

        //Test part way between transition. It should be greater than night brighness. But less than the midpoint because it is not yet sunrise
        let before_sunrise = Utc.ymd(2018, 12, 2).and_hms(7, 50, 0).timestamp();
        let r = sine_curve(before_sunrise, t_secs, rise, false, low, high);
        assert!(r.brightness > low && r.brightness < midpoint); //~41

        //At sunrise it should be half way between the day and night brightness
        let r = sine_curve(rise, t_secs, rise, false, low, high);
        assert_eq!(midpoint, r.brightness); //55.5 is rounded to 56

        //At end of the transition it should equal the daytime brightness
        let end_of_transition = Utc.ymd(2018, 12, 2).and_hms(8, 20, 0).timestamp();
        let r = sine_curve(end_of_transition, t_secs, rise, false, low, high);
        assert_eq!(high, r.brightness); // 76
    }
}
