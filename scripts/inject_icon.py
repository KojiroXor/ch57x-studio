import struct
import ctypes
from ctypes import wintypes
import os
import sys

def inject_icon_into_exe(exe_path, ico_path):
    if not os.path.exists(exe_path):
        print(f"Error: exe file not found: {exe_path}")
        return False
    if not os.path.exists(ico_path):
        print(f"Error: ico file not found: {ico_path}")
        return False

    with open(ico_path, "rb") as f:
        ico_data = f.read()

    # Parse ICONDIR
    reserved, ico_type, count = struct.unpack_from("<HHH", ico_data, 0)
    if reserved != 0 or ico_type != 1 or count == 0:
        print(f"Invalid ICO file: {ico_path}")
        return False

    kernel32 = ctypes.WinDLL("kernel32", use_last_error=True)
    
    kernel32.BeginUpdateResourceW.argtypes = [wintypes.LPCWSTR, wintypes.BOOL]
    kernel32.BeginUpdateResourceW.restype = wintypes.HANDLE

    kernel32.UpdateResourceW.argtypes = [
        wintypes.HANDLE,
        ctypes.c_void_p,
        ctypes.c_void_p,
        wintypes.WORD,
        ctypes.c_char_p,
        wintypes.DWORD,
    ]
    kernel32.UpdateResourceW.restype = wintypes.BOOL

    kernel32.EndUpdateResourceW.argtypes = [wintypes.HANDLE, wintypes.BOOL]
    kernel32.EndUpdateResourceW.restype = wintypes.BOOL

    abs_exe = os.path.abspath(exe_path)
    h_update = kernel32.BeginUpdateResourceW(abs_exe, False)
    if not h_update:
        err = ctypes.get_last_error()
        print(f"BeginUpdateResourceW failed with error code: {err}")
        return False

    RT_ICON = 3
    RT_GROUP_ICON = 14
    LANG_NEUTRAL = 0

    # Build GRPICONDIR
    grp_header = struct.pack("<HHH", 0, 1, count)
    grp_entries = bytearray()

    # Process each icon image
    entry_offset = 6
    for i in range(1, count + 1):
        # Read ICONDIRENTRY (16 bytes)
        entry_bytes = ico_data[entry_offset : entry_offset + 16]
        w, h, colors, res, planes, bpp, bytes_in_res, img_offset = struct.unpack(
            "<BBBBHHII", entry_bytes
        )
        entry_offset += 16

        # Extract icon image data
        img_data = ico_data[img_offset : img_offset + bytes_in_res]

        # UpdateResourceW for RT_ICON (ID = i)
        res = kernel32.UpdateResourceW(
            h_update,
            RT_ICON,
            i,
            LANG_NEUTRAL,
            img_data,
            len(img_data),
        )
        if not res:
            err = ctypes.get_last_error()
            print(f"UpdateResourceW RT_ICON #{i} failed with error: {err}")
            kernel32.EndUpdateResourceW(h_update, True)
            return False

        # Build GRPICONDIRENTRY (14 bytes: 12 bytes header + 2 bytes nID)
        grp_entries += entry_bytes[:12] + struct.pack("<H", i)

    # UpdateResourceW for RT_GROUP_ICON (ID = 1, Windows explorer default app icon)
    grp_data = bytes(grp_header + grp_entries)
    res = kernel32.UpdateResourceW(
        h_update,
        RT_GROUP_ICON,
        1,
        LANG_NEUTRAL,
        grp_data,
        len(grp_data),
    )
    if not res:
        err = ctypes.get_last_error()
        print(f"UpdateResourceW RT_GROUP_ICON failed with error: {err}")
        kernel32.EndUpdateResourceW(h_update, True)
        return False

    # EndUpdateResourceW(hUpdate, fDiscard) -> False means SAVE!
    saved = kernel32.EndUpdateResourceW(h_update, False)
    if not saved:
        err = ctypes.get_last_error()
        print(f"EndUpdateResourceW failed to write changes: {err}")
        return False

    print(f"Successfully embedded {count} icon resolution(s) from '{ico_path}' into '{exe_path}'!")
    return True

if __name__ == "__main__":
    target_exe = sys.argv[1] if len(sys.argv) > 1 else "dist/VortexDeck/vortexdeck.exe"
    target_ico = sys.argv[2] if len(sys.argv) > 2 else "assets/icons/vortexdeck_app_icon.ico"
    success = inject_icon_into_exe(target_exe, target_ico)
    sys.exit(0 if success else 1)
