use crate::driver::TauriDriver;
use anyhow::Result;
use std::path::Path;

pub struct CaptureConfig {
    pub output_dir: String,
    pub view_filter: Option<String>,
}

pub async fn capture_all(driver: &TauriDriver, config: &CaptureConfig) -> Result<Vec<String>> {
    let output_dir = Path::new(&config.output_dir);
    std::fs::create_dir_all(output_dir)?;

    let mut captured = Vec::new();

    // Filter views if specified
    let should_capture = |view: &str| {
        config
            .view_filter
            .as_ref()
            .map_or(true, |f| f == view || f == "all")
    };

    if should_capture("home") {
        captured.extend(capture_home(driver, output_dir).await?);
    }

    if should_capture("settings") {
        captured.extend(capture_settings(driver, output_dir).await?);
    }

    if should_capture("presets") {
        captured.extend(capture_presets(driver, output_dir).await?);
    }

    if should_capture("shortcut") {
        captured.extend(capture_shortcut(driver, output_dir).await?);
    }

    if should_capture("about") {
        captured.extend(capture_about(driver, output_dir).await?);
    }

    Ok(captured)
}

async fn capture_home(driver: &TauriDriver, output_dir: &Path) -> Result<Vec<String>> {
    let mut captured = Vec::new();

    driver.navigate("/").await?;

    let path = output_dir.join("home-idle.png");
    driver.screenshot(path.to_str().unwrap()).await?;
    captured.push("home-idle.png".to_string());

    Ok(captured)
}

async fn capture_settings(driver: &TauriDriver, output_dir: &Path) -> Result<Vec<String>> {
    let mut captured = Vec::new();

    // Helper to save screenshot
    let save = |name: &str| output_dir.join(name);

    // ========== CLOUD MODE ==========
    driver.navigate("/settings").await?;

    // Explicitly ensure cloud mode (click first mode card)
    let _ = driver.click(".mode-card:first-child").await;

    // Cloud default
    driver
        .screenshot(save("settings-cloud-default.png").to_str().unwrap())
        .await?;
    captured.push("settings-cloud-default.png".to_string());

    // Cloud + help panel
    if driver.click(".help-btn").await.is_ok() {
        driver
            .screenshot(save("settings-cloud-help.png").to_str().unwrap())
            .await?;
        captured.push("settings-cloud-help.png".to_string());
        let _ = driver.click(".help-btn").await; // close help
    }

    // Cloud + options expanded
    if driver.click(".options-toggle").await.is_ok() {
        driver
            .screenshot(save("settings-cloud-options.png").to_str().unwrap())
            .await?;
        captured.push("settings-cloud-options.png".to_string());

        // Cloud + polishing enabled (scroll to ensure visible)
        let _ = driver.scroll_to(".toggle-switch").await;
        if driver.click(".toggle-switch").await.is_ok() {
            driver
                .screenshot(save("settings-cloud-polishing.png").to_str().unwrap())
                .await?;
            captured.push("settings-cloud-polishing.png".to_string());

            // Cloud + polishing + Ollama selected
            let _ = driver.scroll_to(".polishing-section .select-trigger").await;
            if driver
                .select_option(".polishing-section .select-trigger", "Ollama")
                .await
                .is_ok()
            {
                driver
                    .screenshot(
                        save("settings-cloud-polishing-ollama.png")
                            .to_str()
                            .unwrap(),
                    )
                    .await?;
                captured.push("settings-cloud-polishing-ollama.png".to_string());
            }

            // Turn off polishing for next section
            let _ = driver.click(".toggle-switch").await;
        }
        // Collapse options
        let _ = driver.click(".options-toggle").await;
    }

    // ========== LOCAL MODE ==========
    // Switch to local mode (click second mode card)
    if driver.click(".mode-card:nth-child(2)").await.is_ok() {
        driver
            .screenshot(save("settings-local-default.png").to_str().unwrap())
            .await?;
        captured.push("settings-local-default.png".to_string());

        // Local + options expanded
        if driver.click(".options-toggle").await.is_ok() {
            driver
                .screenshot(save("settings-local-options.png").to_str().unwrap())
                .await?;
            captured.push("settings-local-options.png".to_string());

            // Local + remote whisper mode
            if driver
                .select_option(
                    ".options-section .field-row:first-child .select-trigger",
                    "Remote",
                )
                .await
                .is_ok()
            {
                driver
                    .screenshot(save("settings-local-remote.png").to_str().unwrap())
                    .await?;
                captured.push("settings-local-remote.png".to_string());
            }

            // Local + polishing enabled (scroll to ensure visible)
            let _ = driver.scroll_to(".toggle-switch").await;
            if driver.click(".toggle-switch").await.is_ok() {
                driver
                    .screenshot(save("settings-local-polishing.png").to_str().unwrap())
                    .await?;
                captured.push("settings-local-polishing.png".to_string());

                // Local + polishing + Ollama
                let _ = driver.scroll_to(".polishing-section .select-trigger").await;
                if driver
                    .select_option(".polishing-section .select-trigger", "Ollama")
                    .await
                    .is_ok()
                {
                    driver
                        .screenshot(
                            save("settings-local-polishing-ollama.png")
                                .to_str()
                                .unwrap(),
                        )
                        .await?;
                    captured.push("settings-local-polishing-ollama.png".to_string());
                }
            }
        }
    }

    Ok(captured)
}

async fn capture_presets(driver: &TauriDriver, output_dir: &Path) -> Result<Vec<String>> {
    let mut captured = Vec::new();

    driver.navigate("/presets").await?;

    let path = output_dir.join("presets-list.png");
    driver.screenshot(path.to_str().unwrap()).await?;
    captured.push("presets-list.png".to_string());

    Ok(captured)
}

async fn capture_shortcut(driver: &TauriDriver, output_dir: &Path) -> Result<Vec<String>> {
    let mut captured = Vec::new();

    driver.navigate("/shortcut").await?;

    let path = output_dir.join("shortcut-current.png");
    driver.screenshot(path.to_str().unwrap()).await?;
    captured.push("shortcut-current.png".to_string());

    Ok(captured)
}

async fn capture_about(driver: &TauriDriver, output_dir: &Path) -> Result<Vec<String>> {
    let mut captured = Vec::new();

    driver.navigate("/about").await?;

    let path = output_dir.join("about.png");
    driver.screenshot(path.to_str().unwrap()).await?;
    captured.push("about.png".to_string());

    Ok(captured)
}
