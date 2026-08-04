# [ GOGIK // AUDIO ]

## GOG // FLT-01

> **PLG-04 // FREQUENCY**  
> *Техно-минимализм для саунд-дизайна. Разработано на чистом Rust.*

![Rust](https://img.shields.io/badge/Made%20with-Rust-black?style=for-the-badge&logo=rust)
![VST3](https://img.shields.io/badge/Format-VST3-blue?style=for-the-badge)
![CLAP](https://img.shields.io/badge/Format-CLAP-red?style=for-the-badge)
![Standalone](https://img.shields.io/badge/Format-Standalone-darkgray?style=for-the-badge)

Многорежимный фильтр для хирургического вмешательства в частотный спектр. Часть экосистемы **GOGIK AUDIO**. Плагин предлагает фильтрацию высоких/низких/полосовых частот с контролем резонанса и сглаживанием 50ms.

---

### ⚙️ ТЕХНИЧЕСКИЕ ОСОБЕННОСТИ

* **DSP Ядро:** Фильтрация высоких/низких/полосовых частот (Low/High/Band-Pass) с контролем резонанса и сглаживанием 50ms.
* **UI & Визуализация:** Потокобезопасные элементы управления и мониторинга выходного сигнала. Отрисовка интерфейса базируется на фреймворке `egui` (через `nih_plug_egui`) с использованием общей библиотеки стилей `gog_common`.
* **Сглаживание (Smoothing):** Линейное сглаживание параметров (`SmoothingStyle::Linear`) с окном 50ms для исключения щелчков при переключении типов волн и частот.

---

### 📦 ФОРМАТЫ И СОВМЕСТИМОСТЬ

Плагин поставляется в следующих форматах (64-bit):

* **VST3** (Windows / macOS)
* **CLAP** (Windows / macOS)
* **Standalone** (Desktop Executable с поддержкой флага `--period-size 2048`)

---

### 🛠 СБОРКА ИЗ ИСХОДНОГО КОДА

Проект использует фреймворк [nih_plug](https://github.com/robbert-vdh/nih-plug) и встроенную утилиту `xtask` для оркестровки сборки бандлов.

**1. Предварительные требования:**

* Установленный `rustup` (актуальная стабильная версия Rust).

**2. Клонирование и сборка:**

```bash
git clone https://github.com/georgejawoods/GOG-FLT-01.git
cd GOG-FLT-01

# Сборка Release-версии (автоматически соберет VST3, CLAP и Standalone)
cargo xtask bundle flt-01 --release
```

Собранные бандлы будут находиться в директории target/bundled/flt-01.vst3.

📜 ЛИЦЕНЗИЯ
Open Source / Внутренняя разработка.

© 2026 GOGIK AUDIO. Все права защищены.
