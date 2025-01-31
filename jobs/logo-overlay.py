import os
import cv2
import numpy as np
from PIL import Image, ImageDraw, ImageFont
from datetime import datetime
import random

# Get hostname and timestamp
hostname = os.uname().nodename
timestamp = datetime.now().strftime("%Y-%m-%d %H:%M:%S")

# Configurations
INPUT_IMAGE = "kluge-logo.png"  # Ensure this image exists in the working directory
OUTPUT_IMAGE = f"kluge_output_{hostname}.png"
FONT_PATH = "./EnvyCodeRNerdFontPropo-Bold.ttf"  # Adjust if needed
FONT_SIZE = 18

# Load image
image = Image.open(INPUT_IMAGE).convert("RGBA")
width, height = image.size

# Rotate image randomly within ±15 degrees
angle = random.uniform(-15, 15)
image = image.rotate(angle, expand=True, resample=Image.BICUBIC)

# Create a black background
background = Image.new("RGBA", image.size, (0, 0, 0, 255))
image = Image.alpha_composite(background, image)

# Create overlay text
overlay = Image.new("RGBA", image.size, (255, 255, 255, 0))
draw = ImageDraw.Draw(overlay)
font = ImageFont.truetype(FONT_PATH, FONT_SIZE)

# Positioning text (bottom left corner)
text = f"{hostname}\n{timestamp}"
bbox = draw.textbbox((0, 0), text, font=font)
text_w, text_h = bbox[2] - bbox[0], bbox[3] - bbox[1]
text_x, text_y = 5, height - text_h - 1
draw.text((text_x, text_y), text, font=font, fill=(255, 0, 0, 255))  # Black text

# Merge overlay with text onto the image
image = Image.alpha_composite(image, overlay)

# Save output
image.convert("RGB").save(OUTPUT_IMAGE)

print(f"Processed image saved as {OUTPUT_IMAGE} on {hostname} at {timestamp}")
