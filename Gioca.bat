@echo off
rem Avvia Companion Survivors con un doppio clic: cargo run dalla cartella del progetto.
rem Le opzioni del gioco si possono aggiungere dopo il nome (es. Gioca.bat --seed 3).
title Companion Survivors
cd /d "%~dp0"
where cargo >nul 2>nul || set "PATH=%USERPROFILE%\.cargo\bin;%PATH%"
where cargo >nul 2>nul || (
    echo Cargo non trovato: installa Rust da https://rustup.rs
    pause
    exit /b 1
)
echo Compilo se serve e avvio il gioco (la prima volta puo' volerci qualche minuto)...
cargo run -- %*
if errorlevel 1 (
    echo.
    echo Il gioco si e' chiuso con un errore: leggi i messaggi qui sopra.
    pause
)
