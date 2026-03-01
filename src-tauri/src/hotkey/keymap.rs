/// Virtual key code lookup table.
///
/// Maps lowercase key name strings (e.g. "a", "f9", "space") to Win32 VK codes.
/// Returns `None` for unrecognised tokens.
use windows::Win32::UI::Input::KeyboardAndMouse::*;

/// Look up the Win32 virtual key code for a key name token.
/// The token must already be trimmed and lowercased.
pub fn token_to_vk(token: &str) -> Option<u32> {
    let code: u32 = match token {
        // Letters
        "a" => VK_A.0 as u32,
        "b" => VK_B.0 as u32,
        "c" => VK_C.0 as u32,
        "d" => VK_D.0 as u32,
        "e" => VK_E.0 as u32,
        "f" => VK_F.0 as u32,
        "g" => VK_G.0 as u32,
        "h" => VK_H.0 as u32,
        "i" => VK_I.0 as u32,
        "j" => VK_J.0 as u32,
        "k" => VK_K.0 as u32,
        "l" => VK_L.0 as u32,
        "m" => VK_M.0 as u32,
        "n" => VK_N.0 as u32,
        "o" => VK_O.0 as u32,
        "p" => VK_P.0 as u32,
        "q" => VK_Q.0 as u32,
        "r" => VK_R.0 as u32,
        "s" => VK_S.0 as u32,
        "t" => VK_T.0 as u32,
        "u" => VK_U.0 as u32,
        "v" => VK_V.0 as u32,
        "w" => VK_W.0 as u32,
        "x" => VK_X.0 as u32,
        "y" => VK_Y.0 as u32,
        "z" => VK_Z.0 as u32,
        // Digits
        "0" => VK_0.0 as u32,
        "1" => VK_1.0 as u32,
        "2" => VK_2.0 as u32,
        "3" => VK_3.0 as u32,
        "4" => VK_4.0 as u32,
        "5" => VK_5.0 as u32,
        "6" => VK_6.0 as u32,
        "7" => VK_7.0 as u32,
        "8" => VK_8.0 as u32,
        "9" => VK_9.0 as u32,
        // Function keys
        "f1" => VK_F1.0 as u32,
        "f2" => VK_F2.0 as u32,
        "f3" => VK_F3.0 as u32,
        "f4" => VK_F4.0 as u32,
        "f5" => VK_F5.0 as u32,
        "f6" => VK_F6.0 as u32,
        "f7" => VK_F7.0 as u32,
        "f8" => VK_F8.0 as u32,
        "f9" => VK_F9.0 as u32,
        "f10" => VK_F10.0 as u32,
        "f11" => VK_F11.0 as u32,
        "f12" => VK_F12.0 as u32,
        // Special keys
        "space" => VK_SPACE.0 as u32,
        "tab" => VK_TAB.0 as u32,
        "escape" | "esc" => VK_ESCAPE.0 as u32,
        "insert" => VK_INSERT.0 as u32,
        "delete" => VK_DELETE.0 as u32,
        "home" => VK_HOME.0 as u32,
        "end" => VK_END.0 as u32,
        "pageup" => VK_PRIOR.0 as u32,
        "pagedown" => VK_NEXT.0 as u32,
        // Arrow keys
        "up" => VK_UP.0 as u32,
        "down" => VK_DOWN.0 as u32,
        "left" => VK_LEFT.0 as u32,
        "right" => VK_RIGHT.0 as u32,
        _ => return None,
    };
    Some(code)
}
