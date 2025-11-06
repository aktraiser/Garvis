#!/usr/bin/env python3
from PIL import Image
import os

def resize_transparent_logo():
    """Redimensionne le logo transparent pour toutes les tailles nécessaires"""
    
    logo_path = "/Users/lucasbometon/Desktop/voice_flow/gravis/gravis-app/src-tauri/icons/garvis_logo.png"
    icons_dir = "/Users/lucasbometon/Desktop/voice_flow/gravis/gravis-app/src-tauri/icons"
    
    print("🎨 Redimensionnement du logo transparent GRAVIS...")
    
    # Charger le logo original
    logo = Image.open(logo_path).convert("RGBA")
    
    # Tailles nécessaires
    sizes = [
        (32, "32x32.png"),
        (128, "128x128.png"), 
        (256, "128x128@2x.png"),
    ]
    
    for size, filename in sizes:
        # Redimensionner avec anti-aliasing
        resized = logo.resize((size, size), Image.Resampling.LANCZOS)
        
        # Sauvegarder
        output_path = os.path.join(icons_dir, filename)
        resized.save(output_path, 'PNG')
        print(f"✅ Créé: {filename} ({size}x{size})")
    
    # Pour .icns et .ico, copier le logo original haute résolution
    import shutil
    
    # Copier pour .icns (macOS)
    shutil.copy(logo_path, os.path.join(icons_dir, "icon.icns"))
    print("✅ Créé: icon.icns (copie haute résolution)")
    
    # Copier pour .ico (Windows) 
    shutil.copy(logo_path, os.path.join(icons_dir, "icon.ico"))
    print("✅ Créé: icon.ico (copie haute résolution)")
    
    print("\n🎉 Redimensionnement terminé ! Logo transparent appliqué à toutes les tailles.")

if __name__ == "__main__":
    resize_transparent_logo()