use std::ffi::CStr;
use std::os::raw::c_char;

#[no_mangle]
pub extern "C" fn spotify_play(track_id: *const c_char) {
    let track_id = unsafe { CStr::from_ptr(track_id).to_string_lossy() };

    println!("Playing track: {}", track_id);
}
