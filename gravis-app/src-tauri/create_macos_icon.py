#!/usr/bin/env python3
from PIL import Image, ImageDraw
import os

def create_macos_compliant_icon():
    """Crée une icône conforme aux standards macOS avec padding transparent"""
    
    logo_path = "/Users/lucasbometon/Desktop/voice_flow/gravis/gravis-app/src-tauri/icons/garvis_logo.png"
    icons_dir = "/Users/lucasbometon/Desktop/voice_flow/gravis/gravis-app/src-tauri/icons"
    
    print("🍎 Création d'icônes conformes aux standards macOS...")
    
    # Charger le logo original
    logo = Image.open(logo_path).convert("RGBA")
    
    def create_icon_with_padding(size, padding_percent=20):
        """Crée une icône avec le bon padding transparent pour macOS"""
        
        # Créer une image transparente de la taille finale
        icon = Image.new('RGBA', (size, size), (0, 0, 0, 0))
        
        # Calculer la taille du logo (80% de l'icône finale)
        logo_size = int(size * (100 - padding_percent) / 100)
        
        # Redimensionner le logo
        logo_resized = logo.resize((logo_size, logo_size), Image.Resampling.LANCZOS)
        
        # Calculer la position pour centrer le logo
        offset = (size - logo_size) // 2
        
        # Coller le logo au centre
        icon.paste(logo_resized, (offset, offset), logo_resized)
        
        return icon
    
    # Tailles nécessaires avec padding approprié (40% = logo 60% de la taille)
    sizes = [
        (32, "32x32.png", 40),
        (128, "128x128.png", 40), 
        (256, "128x128@2x.png", 40),
        (512, "icon.png", 40),  # Icône principale
    ]
    
    for size, filename, padding in sizes:
        icon = create_icon_with_padding(size, padding)
        output_path = os.path.join(icons_dir, filename)
        icon.save(output_path, 'PNG')
        print(f"✅ Créé: {filename} ({size}x{size}) avec {padding}% padding")
    
    # Créer .icns pour macOS avec iconutil
    print("\n🔧 Création du fichier .icns macOS...")
    
    try:
        import subprocess
        
        # Créer un dossier temporaire pour iconset
        iconset_dir = "/tmp/gravis_macos.iconset"
        os.makedirs(iconset_dir, exist_ok=True)
        
        # Générer toutes les tailles requises pour iconset avec plus de padding
        iconset_configs = [
            (16, "icon_16x16.png", 45),
            (32, "icon_16x16@2x.png", 45),
            (32, "icon_32x32.png", 40),
            (64, "icon_32x32@2x.png", 40),
            (128, "icon_128x128.png", 38),
            (256, "icon_128x128@2x.png", 38),
            (256, "icon_256x256.png", 35),
            (512, "icon_256x256@2x.png", 35),
            (512, "icon_512x512.png", 32),
            (1024, "icon_512x512@2x.png", 32),
        ]
        
        for size, filename, padding in iconset_configs:
            icon = create_icon_with_padding(size, padding)
            output_path = os.path.join(iconset_dir, filename)
            icon.save(output_path, 'PNG')
        
        # Convertir en .icns avec iconutil
        icns_path = os.path.join(icons_dir, "icon.icns")
        result = subprocess.run(
            ['iconutil', '-c', 'icns', iconset_dir, '-o', icns_path], 
            capture_output=True, text=True
        )
        
        if result.returncode == 0:
            print(f"✅ Créé: icon.icns (format macOS natif)")
        else:
            print(f"⚠️  Échec création .icns: {result.stderr}")
            # Fallback: utiliser le PNG haute résolution
            fallback_icon = create_icon_with_padding(1024, 12)
            fallback_icon.save(icns_path.replace('.icns', '_fallback.png'), 'PNG')
            print(f"✅ Fallback: créé icon_fallback.png")
        
        # Nettoyer le dossier temporaire
        import shutil
        shutil.rmtree(iconset_dir)
        
    except Exception as e:
        print(f"⚠️  Erreur lors de la création .icns: {e}")
        # Créer un fallback PNG
        fallback_icon = create_icon_with_padding(1024, 12)
        fallback_path = os.path.join(icons_dir, "icon_macos_fallback.png")
        fallback_icon.save(fallback_path, 'PNG')
        print(f"✅ Fallback: créé icon_macos_fallback.png")
    
    # Créer aussi .ico pour Windows avec padding
    print("\n🪟 Création du fichier .ico Windows...")
    ico_sizes = [16, 32, 48, 64, 128, 256]
    ico_images = []
    
    for size in ico_sizes:
        # Plus de padding pour les petites tailles Windows
        padding = 45 if size <= 32 else 40 if size <= 64 else 35
        icon = create_icon_with_padding(size, padding)
        ico_images.append(icon)
    
    ico_path = os.path.join(icons_dir, "icon.ico")
    ico_images[0].save(
        ico_path, 
        format='ICO', 
        sizes=[(img.width, img.height) for img in ico_images]
    )
    print(f"✅ Créé: icon.ico (multi-résolution Windows)")
    
    print("\n🎉 Icônes macOS créées avec succès !")
    print("📏 Chaque icône a maintenant le bon padding transparent pour macOS")
    print("🍎 L'icône devrait maintenant avoir la même taille que les autres apps dans le dock")

if __name__ == "__main__":
    create_macos_compliant_icon()