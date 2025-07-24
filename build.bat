@echo off
echo ========================================
echo Memory-Pak Windows Build Script
echo ========================================

REM Clean previous build
echo Cleaning previous build...
if exist "dist" rmdir /s /q "dist"
if exist "build" rmdir /s /q "build"
if exist "*.spec" del /q "*.spec"
if exist "venv" rmdir /s /q "venv"

REM Check Python installation
echo Checking Python installation...
python --version >nul 2>&1
if errorlevel 1 (
    echo Error: Python is not installed or not in PATH
    pause
    exit /b 1
)

REM Create and activate virtual environment
echo Creating virtual environment...
python -m venv venv
if errorlevel 1 (
    echo Error: Failed to create virtual environment
    pause
    exit /b 1
)

echo Activating virtual environment...
call venv\Scripts\activate.bat
if errorlevel 1 (
    echo Error: Failed to activate virtual environment
    pause
    exit /b 1
)

REM Install dependencies in virtual environment
echo Installing dependencies...
pip install -r requirements.txt
if errorlevel 1 (
    echo Error: Failed to install dependencies
    pause
    exit /b 1
)

REM Create dist directory
if not exist "dist" mkdir "dist"

REM Build the application
echo Building Memory-Pak...
pyinstaller --onefile --windowed --name "Memory-Pak" --icon "icons/switch.png" main.py

REM Check if build was successful
if errorlevel 1 (
    echo Build failed!
    pause
    exit /b 1
)

echo Build completed successfully!

REM Copy data files to dist folder
echo Copying data files...
if exist "consoles.yaml" (
    copy "consoles.yaml" "dist\" >nul
    echo - consoles.yaml copied
) else (
    echo Warning: consoles.yaml not found
)

if exist "games" (
    xcopy "games" "dist\games\" /E /I /Y >nul
    echo - games folder copied
) else (
    echo Warning: games folder not found
)

REM Copy user data files if they exist (for testing)
if exist "user_data.json" (
    copy "user_data.json" "dist\" >nul
    echo - user_data.json copied
)

if exist "settings.json" (
    copy "settings.json" "dist\" >nul
    echo - settings.json copied
)

REM Deactivate virtual environment
echo Deactivating virtual environment...
call venv\Scripts\deactivate.bat

echo ========================================
echo Build completed successfully!
echo.
echo Files created:
echo - dist\Memory-Pak.exe
echo.
echo To run the application:
echo 1. Navigate to the dist folder
echo 2. Double-click Memory-Pak.exe
echo ========================================

REM Show file size
if exist "dist\Memory-Pak.exe" (
    for %%A in ("dist\Memory-Pak.exe") do echo Executable size: %%~zA bytes
)

pause 