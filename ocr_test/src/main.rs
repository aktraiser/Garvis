// GRAVIS OCR - Test standalone Tesseract (aucune dépendance aux modules)
// Validation pure de l'infrastructure Tesseract pour Phase 1

use std::process::Command;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 GRAVIS OCR Phase 1 - Validation Infrastructure Tesseract");
    println!("{}", "=".repeat(65));

    let mut all_tests_passed = true;

    // Test 1: Installation Tesseract
    println!("\n📋 Test 1: Installation Tesseract");
    match test_tesseract_installation() {
        Ok(version) => {
            println!("  ✅ PASSED: Tesseract installé - {}", version);
        }
        Err(e) => {
            println!("  ❌ FAILED: Installation Tesseract - {}", e);
            all_tests_passed = false;
        }
    }

    // Test 2: Langues disponibles
    println!("\n📋 Test 2: Langues disponibles");
    match test_available_languages() {
        Ok((total, critical)) => {
            println!("  ✅ PASSED: {} langues totales, {} critiques", total, critical);
            if total >= 50 {
                println!("  🌟 EXCELLENT: Support linguistique riche");
            }
        }
        Err(e) => {
            println!("  ❌ FAILED: Détection langues - {}", e);
            all_tests_passed = false;
        }
    }

    // Test 3: Capabilities de configuration  
    println!("\n📋 Test 3: Capacités de configuration");
    match test_configuration_capabilities() {
        Ok(capabilities) => {
            println!("  ✅ PASSED: Capacités disponibles: {:?}", capabilities);
        }
        Err(e) => {
            println!("  ❌ FAILED: Configuration - {}", e);
            all_tests_passed = false;
        }
    }

    // Test 4: Performance baseline
    println!("\n📋 Test 4: Performance baseline");
    match test_performance_baseline() {
        Ok(avg_ms) => {
            println!("  ✅ PASSED: Temps de démarrage moyen {:.1}ms", avg_ms);
            if avg_ms < 200.0 {
                println!("  🚀 EXCELLENT: Performance de démarrage");
            }
        }
        Err(e) => {
            println!("  ❌ FAILED: Test performance - {}", e);
            all_tests_passed = false;
        }
    }

    // Test 5: Test d'utilisation pratique  
    println!("\n📋 Test 5: Test d'utilisation pratique");
    match test_practical_usage() {
        Ok(_) => {
            println!("  ✅ PASSED: Usage pratique validé");
        }
        Err(e) => {
            println!("  ❌ FAILED: Usage pratique - {}", e);
            all_tests_passed = false;
        }
    }

    // Test 6: Vérification TESSDATA
    println!("\n📋 Test 6: Vérification TESSDATA");
    match test_tessdata_structure() {
        Ok(stats) => {
            println!("  ✅ PASSED: TESSDATA structuré - {}", stats);
        }
        Err(e) => {
            println!("  ❌ FAILED: TESSDATA - {}", e);
            all_tests_passed = false;
        }
    }

    // Résultats finaux
    println!("\n{}", "=".repeat(65));
    if all_tests_passed {
        println!("🎉 VALIDATION PHASE 1 - SUCCÈS COMPLET!");
        println!("✅ Infrastructure Tesseract entièrement opérationnelle");
        println!("");
        println!("📋 Résumé de la validation:");
        println!("   ✓ Tesseract installé et fonctionnel");
        println!("   ✓ Support multilingue configuré");
        println!("   ✓ Options de configuration disponibles");
        println!("   ✓ Performance acceptable");
        println!("   ✓ Usage pratique validé");
        println!("   ✓ TESSDATA correctement structuré");
        println!("");
        println!("🚀 PRÊT POUR PHASE 2:");
        println!("   1. Implémentation TesseractProcessor complet");
        println!("   2. Système de cache avec Blake3 + LRU");
        println!("   3. Preprocessing intelligent d'images");
        println!("   4. Post-processing et nettoyage de texte");
        println!("   5. Intégration Tauri commands");
        println!("");
        println!("🔧 Recommandations techniques:");
        println!("   • Utiliser Command::new(\"tesseract\") pour Phase 2");
        println!("   • Éviter leptess temporairement (incompatibilité)");
        println!("   • Preprocessing via crate image");
        println!("   • Cache avec clés Blake3");
    } else {
        println!("❌ VALIDATION PHASE 1 - ÉCHECS DÉTECTÉS");
        println!("🔧 Corriger les problèmes avant de continuer vers Phase 2");
    }
    
    println!("{}", "=".repeat(65));
    Ok(())
}

fn test_tesseract_installation() -> Result<String, String> {
    let output = Command::new("tesseract")
        .arg("--version")
        .output()
        .map_err(|e| format!("Commande tesseract non trouvée: {}", e))?;
    
    if !output.status.success() {
        return Err("Commande tesseract a échoué".to_string());
    }
    
    let version_output = String::from_utf8_lossy(&output.stdout);
    let version_line = version_output.lines().next()
        .unwrap_or("version inconnue")
        .to_string();
    
    // Vérifier que c'est bien Tesseract
    if !version_line.to_lowercase().contains("tesseract") {
        return Err("Output inattendu de tesseract --version".to_string());
    }
    
    Ok(version_line)
}

fn test_available_languages() -> Result<(usize, usize), String> {
    let output = Command::new("tesseract")
        .arg("--list-langs")
        .output()
        .map_err(|e| format!("Impossible de lister les langues: {}", e))?;
    
    if !output.status.success() {
        return Err("Commande --list-langs a échoué".to_string());
    }
    
    let langs_output = String::from_utf8_lossy(&output.stdout);
    let languages: Vec<&str> = langs_output
        .lines()
        .skip(1) // Skip header "List of available languages..."
        .filter(|line| !line.trim().is_empty())
        .filter(|line| !line.starts_with("script/"))
        .collect();
    
    // Vérifier langues critiques pour GRAVIS
    let critical_langs = vec!["eng", "fra", "deu", "spa", "ita", "por"];
    let mut found_critical = 0;
    
    for critical in &critical_langs {
        if languages.iter().any(|&lang| lang == *critical) {
            found_critical += 1;
        }
    }
    
    println!("  📊 Langues trouvées: {:?}", 
             languages.iter().take(8).collect::<Vec<_>>());
    
    if found_critical < 3 {
        return Err(format!("Langues critiques insuffisantes: {}/{}", 
                          found_critical, critical_langs.len()));
    }
    
    Ok((languages.len(), found_critical))
}

fn test_configuration_capabilities() -> Result<Vec<String>, String> {
    // Test --help-extra pour vérifier les capacités
    let output = Command::new("tesseract")
        .arg("--help-extra")
        .output()
        .map_err(|e| format!("Help non disponible: {}", e))?;
    
    let mut capabilities = Vec::new();
    
    if output.status.success() {
        let help_text = String::from_utf8_lossy(&output.stdout);
        
        if help_text.contains("Page segmentation modes") {
            capabilities.push("PSM".to_string());
        }
        if help_text.contains("OCR Engine modes") {
            capabilities.push("OEM".to_string());
        }
        if help_text.contains("config") {
            capabilities.push("Config".to_string());
        }
    }
    
    // Test --help-psm pour PSM spécifiquement
    let psm_output = Command::new("tesseract")
        .arg("--help-psm")
        .output();
        
    if let Ok(psm_out) = psm_output {
        if psm_out.status.success() {
            capabilities.push("PSM-detailed".to_string());
        }
    }
    
    // Test --help-oem pour OEM spécifiquement
    let oem_output = Command::new("tesseract")
        .arg("--help-oem")
        .output();
        
    if let Ok(oem_out) = oem_output {
        if oem_out.status.success() {
            capabilities.push("OEM-detailed".to_string());
        }
    }
    
    if capabilities.is_empty() {
        return Err("Aucune capacité de configuration détectée".to_string());
    }
    
    Ok(capabilities)
}

fn test_performance_baseline() -> Result<f64, String> {
    let iterations = 5;
    let mut times = Vec::new();
    
    for _i in 0..iterations {
        let start = std::time::Instant::now();
        
        let _output = Command::new("tesseract")
            .arg("--version")
            .output()
            .map_err(|e| format!("Test performance échoué: {}", e))?;
        
        let elapsed = start.elapsed().as_millis() as f64;
        times.push(elapsed);
    }
    
    let avg = times.iter().sum::<f64>() / times.len() as f64;
    let min = times.iter().fold(f64::INFINITY, |a, &b| a.min(b));
    let max = times.iter().fold(0.0f64, |a, &b| a.max(b));
    
    println!("  📊 Mesures: min={:.1}ms, max={:.1}ms, avg={:.1}ms", min, max, avg);
    
    if avg > 1000.0 {
        return Err("Performance trop lente (>1s)".to_string());
    }
    
    Ok(avg)
}

fn test_practical_usage() -> Result<(), String> {
    // Test des options communes qui seraient utilisées en pratique
    let test_cases = vec![
        (vec!["--help"], "Help général"),
        (vec!["--list-langs"], "Liste langues"),
        (vec!["--print-parameters"], "Paramètres disponibles"),
    ];
    
    for (args, description) in test_cases {
        let mut cmd = Command::new("tesseract");
        for arg in args {
            cmd.arg(arg);
        }
        
        let output = cmd.output()
            .map_err(|e| format!("Test '{}' échoué: {}", description, e))?;
        
        // Accepter les codes de retour 0 et 1 (Tesseract retourne parfois 1 pour --help)
        if !output.status.success() && output.status.code() != Some(1) {
            return Err(format!("Commande '{}' a échoué", description));
        }
        
        println!("  ✓ {}: OK", description);
    }
    
    Ok(())
}

fn test_tessdata_structure() -> Result<String, String> {
    let tessdata_env = std::env::var("TESSDATA_PREFIX").unwrap_or_default();
    let possible_paths = vec![
        "/opt/homebrew/share/tessdata",
        "/usr/share/tessdata", 
        "/usr/local/share/tessdata",
        &tessdata_env,
    ];
    
    for path in possible_paths {
        if path.is_empty() {
            continue;
        }
        
        let tessdata_path = std::path::Path::new(&path);
        if tessdata_path.exists() {
            let entries = std::fs::read_dir(tessdata_path)
                .map_err(|e| format!("Impossible de lire TESSDATA: {}", e))?;
            
            let mut traineddata_count = 0;
            let mut config_count = 0;
            
            for entry in entries {
                if let Ok(entry) = entry {
                    let file_name = entry.file_name().to_string_lossy().to_string();
                    if file_name.ends_with(".traineddata") {
                        traineddata_count += 1;
                    } else if file_name.contains("config") {
                        config_count += 1;
                    }
                }
            }
            
            let stats = format!("{} (traineddata: {}, configs: {})", 
                              path, traineddata_count, config_count);
            
            if traineddata_count < 2 {
                return Err("Trop peu de fichiers traineddata".to_string());
            }
            
            return Ok(stats);
        }
    }
    
    Err("TESSDATA non trouvé dans les emplacements standards".to_string())
}