// test code

#[repr(C)]
pub struct Player {
    volume: i32,
}

#[no_mangle]
pub extern "C" fn player_new_rust() -> *mut Player {
    Box::into_raw(Box::new(Player { volume: 50 }))
}

#[no_mangle]
pub extern "C" fn player_free_rust(player: *mut Player) {
    if player.is_null() { return; }
    unsafe { drop(Box::from_raw(player)) };
}

#[no_mangle]
pub extern "C" fn player_play_rust(_player: *mut Player) {
    println!("Player is playing");
}

#[no_mangle]
pub extern "C" fn player_pause_rust(_player: *mut Player) {
    println!("Player is paused");
}

#[no_mangle]
pub extern "C" fn player_set_volume_rust(player: *mut Player, volume: i32) {
    if let Some(p) = unsafe { player.as_mut() } {
        p.volume = volume;
    }
}

#[no_mangle]
pub extern "C" fn player_get_volume_rust(player: *const Player) -> i32 {
    unsafe { player.as_ref().map_or(0, |p| p.volume) }
}
