import os
from PIL import Image, ImageDraw, ImageFilter
import numpy as np

def create_squircle_mask(size, radius):
    mask = Image.new('L', (size * 4, size * 4), 0)
    draw = ImageDraw.Draw(mask)
    draw.rounded_rectangle([0, 0, size * 4, size * 4], radius=radius * 4, fill=255)
    return mask.resize((size, size), Image.Resampling.LANCZOS)

def generate_app_icon():
    src_tray = "assets/icons/vortexdeck_tray_icon.png"
    if not os.path.exists(src_tray):
        print("Source tray icon not found:", src_tray)
        return

    tray_img = Image.open(src_tray).convert("RGB")
    size = 1024
    
    # 1. Base image (Transparent)
    app_icon = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    
    # Squircle dimensions
    pad = 56
    sq_size = size - (pad * 2)
    radius = int(sq_size * 0.22) # Apple/Fluent squircle radius
    
    # 2. Glass squircle background
    squircle_canvas = Image.new("RGBA", (sq_size, sq_size), (0, 0, 0, 0))
    
    # Gradient background
    y_coords = np.linspace(0, 1, sq_size)
    x_coords = np.linspace(0, 1, sq_size)
    xx, yy = np.meshgrid(x_coords, y_coords)
    
    # Dark luxury obsidian with subtle cyan/indigo tint
    # Top-left to bottom-right gradient
    diag = (xx * 0.4 + yy * 0.6)
    r = (14 * (1 - diag) + 6 * diag).astype(np.uint8)
    g = (20 * (1 - diag) + 10 * diag).astype(np.uint8)
    b = (32 * (1 - diag) + 16 * diag).astype(np.uint8)
    a = np.full((sq_size, sq_size), 245, dtype=np.uint8)
    
    grad_array = np.dstack((r, g, b, a))
    grad_img = Image.fromarray(grad_array, "RGBA")
    
    # Squircle mask
    sq_mask = create_squircle_mask(sq_size, radius)
    squircle_canvas.paste(grad_img, (0, 0), sq_mask)
    
    # 3. Add subtle glowing rim light on border
    border_img = Image.new("RGBA", (sq_size, sq_size), (0, 0, 0, 0))
    b_draw = ImageDraw.Draw(border_img)
    b_draw.rounded_rectangle([2, 2, sq_size - 3, sq_size - 3], radius=radius, outline=(0, 229, 255, 120), width=3)
    b_draw.rounded_rectangle([5, 5, sq_size - 6, sq_size - 6], radius=radius - 3, outline=(0, 255, 200, 40), width=2)
    
    squircle_canvas = Image.alpha_composite(squircle_canvas, border_img)
    
    # 4. Extract glowing vortex from tray_img and make black background transparent
    tray_arr = np.array(tray_img)
    # Brightness estimate
    brightness = np.max(tray_arr, axis=2).astype(np.float32)
    
    # Alpha based on brightness with high contrast
    alpha = np.clip((brightness - 5) * 1.3, 0, 255).astype(np.uint8)
    
    glyph_rgba = np.dstack((tray_arr[:, :, 0], tray_arr[:, :, 1], tray_arr[:, :, 2], alpha))
    glyph_img = Image.fromarray(glyph_rgba, "RGBA")
    
    # 5. Ambient glow behind the glyph
    glow_size = int(sq_size * 0.82)
    ambient_glow = glyph_img.resize((glow_size, glow_size), Image.Resampling.LANCZOS)
    ambient_glow = ambient_glow.filter(ImageFilter.GaussianBlur(radius=40))
    
    # Paste ambient glow centered
    glow_pos = ((sq_size - glow_size) // 2, (sq_size - glow_size) // 2)
    squircle_canvas.paste(ambient_glow, glow_pos, ambient_glow)
    
    # 6. Paste sharp vibrant glyph
    glyph_target_size = int(sq_size * 0.86)
    glyph_sharp = glyph_img.resize((glyph_target_size, glyph_target_size), Image.Resampling.LANCZOS)
    glyph_pos = ((sq_size - glyph_target_size) // 2, (sq_size - glyph_target_size) // 2)
    squircle_canvas.paste(glyph_sharp, glyph_pos, glyph_sharp)
    
    # 7. Subtle top specular glass reflection arc
    specular = Image.new("RGBA", (sq_size, sq_size), (0, 0, 0, 0))
    s_draw = ImageDraw.Draw(specular)
    s_draw.ellipse([-sq_size * 0.2, -sq_size * 0.6, sq_size * 1.2, sq_size * 0.45], fill=(255, 255, 255, 18))
    # Clip specular to squircle
    specular.putalpha(Image.fromarray(np.minimum(np.array(specular.split()[3]), np.array(sq_mask))))
    squircle_canvas = Image.alpha_composite(squircle_canvas, specular)
    
    # 8. Drop shadow for the squircle on the 1024x1024 canvas
    shadow = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    s_mask = create_squircle_mask(sq_size, radius)
    shadow_sq = Image.new("RGBA", (sq_size, sq_size), (0, 0, 0, 180))
    shadow.paste(shadow_sq, (pad, pad + 14), s_mask)
    shadow = shadow.filter(ImageFilter.GaussianBlur(radius=24))
    
    app_icon = Image.alpha_composite(app_icon, shadow)
    app_icon.paste(squircle_canvas, (pad, pad), squircle_canvas)
    
    out_png = "assets/icons/vortexdeck_app_icon.png"
    app_icon.save(out_png, "PNG")
    print(f"Generated new app icon: {out_png} ({size}x{size})")

    # Generate multi-res ICO for App Icon
    ico_sizes = [(16, 16), (24, 24), (32, 32), (48, 48), (64, 64), (128, 128), (256, 256)]
    out_ico = "assets/icons/vortexdeck_app_icon.ico"
    app_icon.save(out_ico, format="ICO", sizes=ico_sizes)
    print(f"Generated new app ico: {out_ico}")

    # Export RGBA for egui/eframe 64x64
    app_64 = app_icon.resize((64, 64), Image.Resampling.LANCZOS)
    with open("assets/icons/vortexdeck_app_icon_64.rgba", "wb") as f:
        f.write(app_64.tobytes("raw", "RGBA"))
    print("Exported: assets/icons/vortexdeck_app_icon_64.rgba (64x64)")

    # Also optimize tray icons to have clean alpha for Windows taskbar
    def process_tray_icon(src_path, out_ico_path, out_rgba_path):
        src = Image.open(src_path).convert("RGB")
        src_arr = np.array(src)
        b_val = np.max(src_arr, axis=2).astype(np.float32)
        alpha_val = np.clip((b_val - 3) * 1.25, 0, 255).astype(np.uint8)
        rgba_arr = np.dstack((src_arr[:, :, 0], src_arr[:, :, 1], src_arr[:, :, 2], alpha_val))
        clean_img = Image.fromarray(rgba_arr, "RGBA")
        
        # Save ICO
        tray_sizes = [(16, 16), (20, 20), (24, 24), (32, 32), (48, 48), (64, 64)]
        clean_img.save(out_ico_path, format="ICO", sizes=tray_sizes)
        print(f"Generated tray ico: {out_ico_path}")
        
        # Save 32x32 RGBA
        tray_32 = clean_img.resize((32, 32), Image.Resampling.LANCZOS)
        with open(out_rgba_path, "wb") as f:
            f.write(tray_32.tobytes("raw", "RGBA"))
        print(f"Exported RGBA: {out_rgba_path} (32x32)")

    process_tray_icon("assets/icons/vortexdeck_tray_icon.png", "assets/icons/vortexdeck_tray_icon.ico", "assets/icons/vortexdeck_tray_icon_32.rgba")
    process_tray_icon("assets/icons/vortexdeck_tray_paused.png", "assets/icons/vortexdeck_tray_paused.ico", "assets/icons/vortexdeck_tray_paused_32.rgba")

if __name__ == "__main__":
    generate_app_icon()
