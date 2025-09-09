#!/usr/bin/env python3
"""
Create a professional app icon for Portify.
"""

from PIL import Image, ImageDraw, ImageFont
import os


def create_professional_icon():
    """Create a professional-looking app icon."""
    
    # Icon sizes for macOS
    sizes = [16, 32, 64, 128, 256, 512, 1024]
    
    icons_dir = "portify/menubar/icons"
    os.makedirs(icons_dir, exist_ok=True)
    
    for size in sizes:
        # Create image with transparent background
        img = Image.new('RGBA', (size, size), (0, 0, 0, 0))
        draw = ImageDraw.Draw(img)
        
        # Modern gradient background (blue to purple)
        for y in range(size):
            # Gradient from blue to purple
            blue_ratio = 1 - (y / size)
            purple_ratio = y / size
            
            r = int(59 * blue_ratio + 147 * purple_ratio)    # Blue to Purple
            g = int(130 * blue_ratio + 51 * purple_ratio)    # Blue to Purple  
            b = int(246 * blue_ratio + 234 * purple_ratio)   # Blue to Purple
            
            draw.line([(0, y), (size, y)], fill=(r, g, b, 255))
        
        # Add rounded corners
        mask = Image.new('L', (size, size), 0)
        mask_draw = ImageDraw.Draw(mask)
        corner_radius = size // 8
        mask_draw.rounded_rectangle([0, 0, size, size], corner_radius, fill=255)
        
        # Apply mask
        img.putalpha(mask)
        
        # Add "P" text
        try:
            # Try to use a nice font
            font_size = size // 2
            font = ImageFont.truetype('/System/Library/Fonts/Helvetica.ttc', font_size)
        except:
            font = ImageFont.load_default()
            font_size = size // 3
        
        # Draw "P" in white
        text = "P"
        bbox = draw.textbbox((0, 0), text, font=font)
        text_width = bbox[2] - bbox[0]
        text_height = bbox[3] - bbox[1]
        
        x = (size - text_width) // 2
        y = (size - text_height) // 2 - size // 20  # Slight adjustment
        
        # Add shadow
        shadow_offset = max(1, size // 64)
        draw.text((x + shadow_offset, y + shadow_offset), text, fill=(0, 0, 0, 100), font=font)
        
        # Draw main text
        draw.text((x, y), text, fill=(255, 255, 255, 255), font=font)
        
        # Save icon
        icon_path = f"{icons_dir}/portify_icon_{size}x{size}.png"
        img.save(icon_path)
        print(f"Created icon: {icon_path}")
    
    # Create .icns file for macOS
    create_icns_file(icons_dir)


def create_icns_file(icons_dir):
    """Create .icns file from PNG icons."""
    try:
        import subprocess
        
        # Create iconset directory
        iconset_dir = f"{icons_dir}/portify.iconset"
        os.makedirs(iconset_dir, exist_ok=True)
        
        # Copy icons with proper naming for iconset
        icon_mappings = {
            16: "icon_16x16.png",
            32: ["icon_16x16@2x.png", "icon_32x32.png"],
            64: "icon_32x32@2x.png", 
            128: ["icon_64x64@2x.png", "icon_128x128.png"],
            256: ["icon_128x128@2x.png", "icon_256x256.png"],
            512: ["icon_256x256@2x.png", "icon_512x512.png"],
            1024: "icon_512x512@2x.png"
        }
        
        for size, names in icon_mappings.items():
            source = f"{icons_dir}/portify_icon_{size}x{size}.png"
            if os.path.exists(source):
                if isinstance(names, list):
                    for name in names:
                        dest = f"{iconset_dir}/{name}"
                        subprocess.run(['cp', source, dest], check=True)
                else:
                    dest = f"{iconset_dir}/{names}"
                    subprocess.run(['cp', source, dest], check=True)
        
        # Create .icns file
        icns_path = f"{icons_dir}/portify.icns"
        result = subprocess.run([
            'iconutil', '-c', 'icns', iconset_dir, '-o', icns_path
        ], capture_output=True, text=True)
        
        if result.returncode == 0:
            print(f"✅ Created .icns file: {icns_path}")
            
            # Clean up iconset directory
            subprocess.run(['rm', '-rf', iconset_dir])
        else:
            print(f"⚠️  Could not create .icns file: {result.stderr}")
            
    except Exception as e:
        print(f"⚠️  Could not create .icns file: {e}")


if __name__ == "__main__":
    print("🎨 Creating Professional App Icon...")
    create_professional_icon()
    print("✅ App icon created successfully!")
