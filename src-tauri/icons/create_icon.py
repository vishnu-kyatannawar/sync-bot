#!/usr/bin/env python3
import zlib
import struct

def create_rgba_png(filename, width, height, r=74, g=144, b=226, a=255):
    """Create a simple RGBA PNG"""
    # PNG signature
    png = b'\x89PNG\r\n\x1a\n'
    
    # Create pixel data (RGBA for each pixel)
    pixel = struct.pack('BBBB', r, g, b, a)
    scanline = b'\x00' + pixel * width  # Filter byte + pixels
    image_data = scanline * height
    
    # Compress image data
    compressed = zlib.compress(image_data)
    
    # IHDR chunk
    ihdr = struct.pack('>IIBBBBB', width, height, 8, 6, 0, 0, 0)
    ihdr_crc = zlib.crc32(b'IHDR' + ihdr) & 0xffffffff
    png += struct.pack('>I', 13) + b'IHDR' + ihdr + struct.pack('>I', ihdr_crc)
    
    # IDAT chunk
    idat_crc = zlib.crc32(b'IDAT' + compressed) & 0xffffffff
    png += struct.pack('>I', len(compressed)) + b'IDAT' + compressed + struct.pack('>I', idat_crc)
    
    # IEND chunk
    png += struct.pack('>I', 0) + b'IEND' + struct.pack('>I', 0xAE426082)
    
    with open(filename, 'wb') as f:
        f.write(png)

create_rgba_png('icon.png', 512, 512)
print("Created RGBA icon.png")
