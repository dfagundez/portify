"""
Icon generator for Portify menu bar app.
Creates simple icons programmatically.
"""

from PIL import Image, ImageDraw, ImageFont
import os


def create_icon(text: str, color: str, size: int = 22) -> Image.Image:
    """Create a simple text-based icon."""
    # Create image with transparent background
    img = Image.new('RGBA', (size, size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)
    
    # Color mapping
    colors = {
        'green': (34, 197, 94, 255),    # Green-500
        'yellow': (234, 179, 8, 255),   # Yellow-500
        'red': (239, 68, 68, 255),      # Red-500
        'blue': (59, 130, 246, 255),    # Blue-500
        'gray': (107, 114, 128, 255),   # Gray-500
    }
    
    fill_color = colors.get(color, colors['gray'])
    
    # Try to use a system font, fallback to default
    try:
        # macOS system font
        font = ImageFont.truetype('/System/Library/Fonts/Helvetica.ttc', size - 4)
    except:
        try:
            # Fallback font
            font = ImageFont.load_default()
        except:
            font = None
    
    # Calculate text position to center it
    if font:
        bbox = draw.textbbox((0, 0), text, font=font)
        text_width = bbox[2] - bbox[0]
        text_height = bbox[3] - bbox[1]
    else:
        text_width = len(text) * 6
        text_height = 10
    
    x = (size - text_width) // 2
    y = (size - text_height) // 2
    
    # Draw text
    draw.text((x, y), text, fill=fill_color, font=font)
    
    return img


def create_portify_icon(status: str = 'normal') -> Image.Image:
    """Create the main Portify icon based on status."""
    size = 22
    
    # Status color mapping
    status_colors = {
        'normal': 'blue',
        'active': 'green', 
        'warning': 'yellow',
        'error': 'red',
        'inactive': 'gray'
    }
    
    color = status_colors.get(status, 'blue')
    
    # Create icon with "P" for Portify
    return create_icon('P', color, size)


def save_icons():
    """Generate and save all required icons."""
    icons_dir = os.path.join(os.path.dirname(__file__), 'icons')
    os.makedirs(icons_dir, exist_ok=True)
    
    # Create different status icons
    statuses = ['normal', 'active', 'warning', 'error', 'inactive']
    
    for status in statuses:
        icon = create_portify_icon(status)
        icon_path = os.path.join(icons_dir, f'portify_{status}.png')
        icon.save(icon_path)
        print(f"Created icon: {icon_path}")
    
    # Create a template icon (for dark mode compatibility)
    template_icon = create_portify_icon('normal')
    template_path = os.path.join(icons_dir, 'portify_template.png')
    template_icon.save(template_path)
    print(f"Created template icon: {template_path}")


if __name__ == "__main__":
    save_icons()
