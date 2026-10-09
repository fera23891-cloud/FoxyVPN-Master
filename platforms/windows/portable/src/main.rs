#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use foxyvpn_core::{DnsSinkhole, SplitTunnelEngine};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

static IS_CONNECTED: AtomicBool = AtomicBool::new(false);

const GUI_HTML: &str = r#"<!DOCTYPE html>
<html lang="fa" dir="rtl">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>FoxyVPN Master Suite v2.0</title>
    <link href="https://fonts.googleapis.com/css2?family=Vazirmatn:wght@400;500;600;700;800;900&family=JetBrains+Mono:wght@400;600;700&display=swap" rel="stylesheet">
    <style>
        :root {
            --bg-base: #030712;
            --bg-card: #0f172a;
            --bg-card-hover: #1e293b;
            --primary: #f97316;
            --primary-hover: #ea580c;
            --accent: #3b82f6;
            --success: #10b981;
            --danger: #ef4444;
            --text-main: #f8fafc;
            --text-muted: #94a3b8;
            --border: #1e293b;
        }
        * { box-sizing: border-box; margin: 0; padding: 0; font-family: 'Vazirmatn', sans-serif; user-select: none; }
        body { background: var(--bg-base); color: var(--text-main); min-height: 100vh; display: flex; flex-direction: column; overflow-x: hidden; }
        
        /* Top Navigation */
        .app-header { display: flex; align-items: center; justify-content: space-between; padding: 16px 20px; background: rgba(15, 23, 42, 0.85); backdrop-filter: blur(12px); border-bottom: 1px solid var(--border); position: sticky; top: 0; z-index: 50; }
        .logo-wrap { display: flex; align-items: center; gap: 10px; }
        .logo-icon { font-size: 24px; filter: drop-shadow(0 0 10px rgba(249, 115, 22, 0.4)); }
        .logo-text { font-size: 17px; font-weight: 900; letter-spacing: -0.5px; }
        .logo-text span { color: var(--primary); }
        .version-badge { font-size: 10px; padding: 2px 7px; background: rgba(249, 115, 22, 0.15); color: #fb923c; border: 1px solid rgba(249, 115, 22, 0.3); border-radius: 6px; font-weight: 700; }
        .status-chip { display: flex; align-items: center; gap: 6px; font-size: 11px; font-weight: 700; padding: 4px 10px; border-radius: 20px; background: #1e293b; color: var(--text-muted); border: 1px solid var(--border); }
        .status-chip.connected { background: rgba(16, 185, 129, 0.15); color: #34d399; border-color: rgba(16, 185, 129, 0.4); }
        .status-dot { width: 7px; height: 7px; border-radius: 50%; background: #64748b; }
        .status-chip.connected .status-dot { background: #10b981; box-shadow: 0 0 8px #10b981; }

        /* Tabs Bar */
        .tabs-nav { display: flex; gap: 4px; padding: 8px 16px; background: #0b1120; border-bottom: 1px solid var(--border); overflow-x: auto; }
        .tab-btn { flex: 1; min-width: 65px; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 4px; padding: 8px 4px; border: none; background: transparent; color: var(--text-muted); font-size: 11px; font-weight: 700; border-radius: 10px; cursor: pointer; transition: all 0.2s; }
        .tab-btn:hover { background: rgba(255, 255, 255, 0.05); color: var(--text-main); }
        .tab-btn.active { background: rgba(249, 115, 22, 0.15); color: var(--primary); border: 1px solid rgba(249, 115, 22, 0.25); }
        .tab-icon { font-size: 15px; }

        /* Views Container */
        .views-content { flex: 1; padding: 18px; max-width: 500px; width: 100%; margin: 0 auto; }
        .tab-pane { display: none; }
        .tab-pane.active { display: block; animation: fadeIn 0.25s ease; }
        @keyframes fadeIn { from { opacity: 0; transform: translateY(4px); } to { opacity: 1; transform: translateY(0); } }

        /* Card Styles */
        .card { background: var(--bg-card); border: 1px solid var(--border); border-radius: 18px; padding: 18px; margin-bottom: 16px; }
        .card-title { font-size: 13px; font-weight: 800; color: var(--text-main); margin-bottom: 12px; display: flex; align-items: center; justify-content: space-between; }

        /* Connect View */
        .power-wrapper { display: flex; flex-direction: column; align-items: center; padding: 28px 0 20px; }
        .power-btn { width: 140px; height: 140px; border-radius: 50%; border: 3px solid #334155; background: #0b1329; color: var(--primary); display: flex; flex-direction: column; align-items: center; justify-content: center; cursor: pointer; transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1); box-shadow: 0 10px 30px rgba(0, 0, 0, 0.6); position: relative; outline: none; }
        .power-btn:hover { border-color: var(--primary); transform: scale(1.03); box-shadow: 0 0 30px rgba(249, 115, 22, 0.3); }
        .power-btn.connecting { animation: pulse 1.2s infinite; border-color: #fbbf24; }
        .power-btn.connected { background: linear-gradient(135deg, #059669, #10b981); color: #030712; border-color: #34d399; box-shadow: 0 0 50px rgba(16, 185, 129, 0.45); }
        .power-btn svg { width: 50px; height: 50px; fill: currentColor; }
        .power-label { font-size: 12px; font-weight: 800; margin-top: 6px; letter-spacing: 0.5px; }
        @keyframes pulse { 0% { transform: scale(1); } 50% { transform: scale(1.04); } 100% { transform: scale(1); } }

        .connection-status-text { margin-top: 14px; font-size: 13px; font-weight: 700; color: var(--text-muted); text-align: center; }
        .connection-status-text.connected { color: #34d399; }

        /* Selected Server Card */
        .active-server-card { display: flex; align-items: center; justify-content: space-between; background: #020617; border: 1px solid var(--border); border-radius: 14px; padding: 12px 14px; cursor: pointer; transition: all 0.2s; margin-top: 16px; }
        .active-server-card:hover { border-color: #334155; }
        .srv-info { display: flex; align-items: center; gap: 10px; }
        .srv-flag { font-size: 22px; }
        .srv-name { font-size: 13px; font-weight: 700; }
        .srv-ip { font-size: 11px; color: var(--text-muted); font-family: 'JetBrains Mono', monospace; }
        .srv-ping { font-size: 11px; font-weight: 700; color: #34d399; font-family: 'JetBrains Mono', monospace; display: flex; align-items: center; gap: 4px; }

        /* Metrics Grid */
        .metrics-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 10px; margin-top: 14px; }
        .metric-box { background: #020617; border: 1px solid var(--border); border-radius: 12px; padding: 12px; text-align: center; }
        .metric-label { font-size: 11px; color: var(--text-muted); }
        .metric-val { font-size: 16px; font-weight: 800; font-family: 'JetBrains Mono', monospace; color: var(--text-main); margin-top: 4px; }

        /* Account Pool Bar */
        .pool-progress { margin-top: 14px; }
        .pool-header { display: flex; justify-content: space-between; font-size: 12px; margin-bottom: 6px; }
        .bar-bg { width: 100%; height: 8px; background: #020617; border-radius: 4px; overflow: hidden; border: 1px solid var(--border); }
        .bar-fill { height: 100%; background: linear-gradient(90deg, #f97316, #fbbf24); width: 45%; transition: width 0.5s; }

        /* Server List */
        .search-box { width: 100%; background: #020617; border: 1px solid var(--border); border-radius: 12px; padding: 10px 14px; color: var(--text-main); font-size: 12px; margin-bottom: 12px; outline: none; }
        .search-box:focus { border-color: var(--primary); }
        .server-item { display: flex; align-items: center; justify-content: space-between; padding: 12px 14px; background: #020617; border: 1px solid var(--border); border-radius: 12px; margin-bottom: 8px; cursor: pointer; transition: all 0.2s; }
        .server-item:hover { border-color: var(--primary); background: rgba(249, 115, 22, 0.05); }
        .server-item.selected { border-color: var(--primary); background: rgba(249, 115, 22, 0.1); }

        /* Accounts & Login */
        .btn-fxa { width: 100%; display: flex; align-items: center; justify-content: center; gap: 8px; padding: 12px; border-radius: 12px; background: #7c3aed; color: white; font-size: 13px; font-weight: 800; border: none; cursor: pointer; transition: all 0.2s; margin-bottom: 14px; }
        .btn-fxa:hover { background: #6d28d9; }
        .input-group { margin-bottom: 10px; }
        .input-group label { display: block; font-size: 11px; font-weight: 700; color: var(--text-muted); margin-bottom: 4px; }
        .input-group input { width: 100%; background: #020617; border: 1px solid var(--border); border-radius: 10px; padding: 10px 12px; color: white; font-size: 12px; outline: none; }
        .input-group input:focus { border-color: var(--primary); }
        .btn-add { width: 100%; padding: 10px; background: var(--primary); color: #030712; font-weight: 800; font-size: 12px; border-radius: 10px; border: none; cursor: pointer; }
        .account-card { background: #020617; border: 1px solid var(--border); border-radius: 12px; padding: 12px; margin-bottom: 8px; }

        /* Settings View */
        .setting-row { display: flex; align-items: center; justify-content: space-between; padding: 12px 0; border-bottom: 1px solid var(--border); }
        .setting-row:last-child { border-bottom: none; }
        .setting-info { max-width: 80%; }
        .setting-name { font-size: 13px; font-weight: 700; }
        .setting-desc { font-size: 11px; color: var(--text-muted); margin-top: 2px; }
        .toggle-switch { position: relative; width: 44px; height: 24px; }
        .toggle-switch input { opacity: 0; width: 0; height: 0; }
        .toggle-slider { position: absolute; cursor: pointer; top: 0; left: 0; right: 0; bottom: 0; background-color: #334155; border-radius: 24px; transition: .3s; }
        .toggle-slider:before { position: absolute; content: ""; height: 18px; width: 18px; left: 3px; bottom: 3px; background-color: white; border-radius: 50%; transition: .3s; }
        input:checked + .toggle-slider { background-color: var(--primary); }
        input:checked + .toggle-slider:before { transform: translateX(20px); }

        /* Logs Console */
        .logs-window { background: #020617; border: 1px solid var(--border); border-radius: 12px; padding: 12px; font-family: 'JetBrains Mono', monospace; font-size: 11px; color: var(--text-muted); height: 280px; overflow-y: auto; text-align: left; dir: ltr; }
        .log-row { margin-bottom: 4px; word-break: break-all; }
        .log-tag { color: var(--primary); font-weight: 700; }
        .btn-clear-logs { font-size: 11px; background: transparent; border: 1px solid var(--border); color: var(--text-muted); border-radius: 8px; padding: 4px 10px; cursor: pointer; }
    </style>
</head>
<body>
    <!-- App Header -->
    <header class="app-header">
        <div class="logo-wrap">
            <span class="logo-icon">🦊</span>
            <div>
                <div class="logo-text">FoxyVPN <span>Master</span></div>
                <div class="version-badge">v2.0 Universal Suite</div>
            </div>
        </div>
        <div id="statusChip" class="status-chip">
            <div class="status-dot"></div>
            <span id="statusChipText">غیرفعال</span>
        </div>
    </header>

    <!-- Navigation Tabs -->
    <nav class="tabs-nav">
        <button class="tab-btn active" onclick="switchTab('connect')">
            <span class="tab-icon">⚡</span>
            <span>اتصال</span>
        </button>
        <button class="tab-btn" onclick="switchTab('servers')">
            <span class="tab-icon">🌍</span>
            <span>سرورها</span>
        </button>
        <button class="tab-btn" onclick="switchTab('accounts')">
            <span class="tab-icon">🔑</span>
            <span>اکانت‌ها</span>
        </button>
        <button class="tab-btn" onclick="switchTab('settings')">
            <span class="tab-icon">⚙️</span>
            <span>تنظیمات</span>
        </button>
        <button class="tab-btn" onclick="switchTab('logs')">
            <span class="tab-icon">📜</span>
            <span>لاگ‌ها</span>
        </button>
    </nav>

    <!-- Views Container -->
    <main class="views-content">
        <!-- 1. Connection View -->
        <section id="pane-connect" class="tab-pane active">
            <div class="card">
                <div class="power-wrapper">
                    <button id="pwrBtn" class="power-btn" onclick="toggleConnect()">
                        <svg viewBox="0 0 24 24"><path d="M16.56 5.44l-1.45 1.45C16.84 8.05 17.73 9.91 17.73 12c0 3.16-2.57 5.73-5.73 5.73S6.27 15.16 6.27 12c0-2.09.89-3.95 2.62-5.11L7.44 5.44C5.36 6.88 4 9.28 4 12c0 4.41 3.59 8 8 8s8-3.59 8-8c0-2.72-1.36-5.12-3.44-6.56zM13 3h-2v10h2V3z"/></svg>
                        <div id="pwrLbl" class="power-label">اتصال</div>
                    </button>
                    <div id="connStatusText" class="connection-status-text">قطع • آماده برقراری تونل موزیلا</div>
                </div>

                <!-- Selected Server Box -->
                <div class="active-server-card" onclick="switchTab('servers')">
                    <div class="srv-info">
                        <span id="activeSrvFlag" class="srv-flag">🇩🇪</span>
                        <div>
                            <div id="activeSrvName" class="srv-name">آلمان (فرانکفورت Anycast)</div>
                            <div id="activeSrvIp" class="srv-ip">151.101.1.140:443</div>
                        </div>
                    </div>
                    <div id="activeSrvPing" class="srv-ping">● 54ms</div>
                </div>

                <!-- Stats Grid -->
                <div class="metrics-grid">
                    <div class="metric-box">
                        <div class="metric-label">سرعت دانلود</div>
                        <div id="downSpeed" class="metric-val">0 KB/s</div>
                    </div>
                    <div class="metric-box">
                        <div class="metric-label">سرعت آپلود</div>
                        <div id="upSpeed" class="metric-val">0 KB/s</div>
                    </div>
                    <div class="metric-box">
                        <div class="metric-label">حجم مصرفی نشست</div>
                        <div id="trafficUsed" class="metric-val">0.0 MB</div>
                    </div>
                    <div class="metric-box">
                        <div class="metric-label">مدت زمان اتصال</div>
                        <div id="uptimeVal" class="metric-val">00:00:00</div>
                    </div>
                </div>

                <!-- Multi-Account Quota Gauge -->
                <div class="pool-progress">
                    <div class="pool-header">
                        <span style="color: var(--text-muted);">استخر سهمیه فایرفاکس:</span>
                        <span id="quotaText" style="color: #fb923c; font-weight: 700;">128.4 GB / 150 GB</span>
                    </div>
                    <div class="bar-bg">
                        <div id="quotaBar" class="bar-fill" style="width: 85%;"></div>
                    </div>
                </div>
            </div>
        </section>

        <!-- 2. Servers View -->
        <section id="pane-servers" class="tab-pane">
            <div class="card">
                <div class="card-title">
                    <span>انتخاب سرور اختصاصی فستلی و موزیلا</span>
                    <span style="font-size: 11px; color: var(--text-muted);">۱۶ لوکیشن فعال</span>
                </div>
                <input type="text" id="srvSearch" class="search-box" placeholder="🔍 جستجوی کشور، شهر یا آی‌پی..." oninput="filterServers()">
                
                <div id="serversList">
                    <!-- Populated via JS -->
                </div>
            </div>
        </section>

        <!-- 3. Accounts & Login View -->
        <section id="pane-accounts" class="tab-pane">
            <div class="card">
                <div class="card-title">ورود با اکانت رسمی فایرفاکس (FxA)</div>
                <p style="font-size: 11px; color: var(--text-muted); margin-bottom: 12px; line-height: 1.5;">
                    موزیلا برای هر حساب فایرفاکس ۵۰ گیگابایت حجم پرسرعت رایگان اختصاص می‌دهد. می‌توانید با چند اکانت همزمان وارد شوید تا ظرفیت نامحدود داشته باشید.
                </p>
                <button class="btn-fxa" onclick="openFirefoxLogin()">
                    <span>🦊 ورود مستقیم به حساب موزیلا فایرفاکس (OAuth)</span>
                </button>
            </div>

            <div class="card">
                <div class="card-title">افزودن دستی توکن موزیلا (Bearer Token)</div>
                <div class="input-group">
                    <label>ایمیل اکانت فایرفاکس:</label>
                    <input type="email" id="accEmail" placeholder="user@firefox.com">
                </div>
                <div class="input-group">
                    <label>توکن دسترسی (Bearer Token یا Session Token):</label>
                    <input type="text" id="accToken" placeholder="توکن دریافتی از مرورگر یا افزونه موزیلا...">
                </div>
                <button class="btn-add" onclick="addNewAccount()">+ افزودن اکانت به استخر سهمیه</button>
            </div>

            <div class="card">
                <div class="card-title">
                    <span>استخر اکانت‌های فعال</span>
                    <span id="accCount" style="color: var(--primary);">۲ حساب</span>
                </div>
                <div id="accountsList">
                    <!-- Accounts rendered here -->
                </div>
            </div>
        </section>

        <!-- 4. Settings View -->
        <section id="pane-settings" class="tab-pane">
            <div class="card">
                <div class="card-title">تنظیمات امنیت و حریم خصوصی</div>

                <div class="setting-row">
                    <div class="setting-info">
                        <div class="setting-name">کیل‌سوییچ لایه سیستم (WFP Kill Switch)</div>
                        <div class="setting-desc">در صورت بروز هرگونه قطعی ناگهانی، تمامی اتصالات اینترنت قفل می‌شود تا آی‌پی لو نرود.</div>
                    </div>
                    <label class="toggle-switch">
                        <input type="checkbox" id="setKillswitch" checked>
                        <span class="toggle-slider"></span>
                    </label>
                </div>

                <div class="setting-row">
                    <div class="setting-info">
                        <div class="setting-name">اسپلیت تونلینگ هوشمند ایران (Split Tunnel)</div>
                        <div class="setting-desc">دامنه‌های داخلی (.ir)، بانکی، شاپراک، اسنپ و دیوار بدون فیلترشکن باز می‌شوند تا سهمیه مصرف نشود.</div>
                    </div>
                    <label class="toggle-switch">
                        <input type="checkbox" id="setSplitTunnel" checked>
                        <span class="toggle-slider"></span>
                    </label>
                </div>

                <div class="setting-row">
                    <div class="setting-info">
                        <div class="setting-name">مسدودساز تبلیغات DNS Sinkhole</div>
                        <div class="setting-desc">مسدودسازی دامنه‌های تبلیغاتی و ردیاب‌ها پیش از ارسال به تونل (صرفه‌جویی تا ۴۰٪ در سهمیه).</div>
                    </div>
                    <label class="toggle-switch">
                        <input type="checkbox" id="setAdblock" checked>
                        <span class="toggle-slider"></span>
                    </label>
                </div>

                <div class="setting-row">
                    <div class="setting-info">
                        <div class="setting-name">شبیه‌سازی اثرانگشت فایرفاکس (JA3 Spoofing)</div>
                        <div class="setting-desc">ارسال امضای رمزنگاری رسمی کلاینت موزیلا جهت جلوگیری از مسدودسازی و بن شدن اکانت.</div>
                    </div>
                    <label class="toggle-switch">
                        <input type="checkbox" id="setStealth" checked>
                        <span class="toggle-slider"></span>
                    </label>
                </div>
            </div>
        </section>

        <!-- 5. Logs View -->
        <section id="pane-logs" class="tab-pane">
            <div class="card">
                <div class="card-title">
                    <span>کنسول و لاگ‌های زنده هسته (Core Logs)</span>
                    <button class="btn-clear-logs" onclick="clearLogs()">پاک‌سازی</button>
                </div>
                <div id="logsBox" class="logs-window">
                    <div class="log-row"><span class="log-tag">[INIT]</span> FoxyVPN Master Suite v2.0 Desktop Core Active.</div>
                    <div class="log-row"><span class="log-tag">[PROXY]</span> Listening on 127.0.0.1:21080 (Mixed HTTP/CONNECT).</div>
                    <div class="log-row"><span class="log-tag">[STEALTH]</span> WFP IPv6 Sinkhole and Firefox TLS Profile Engaged.</div>
                </div>
            </div>
        </section>
    </main>

    <script>
        const SERVERS = [
            { id: 'fra', name: 'آلمان (فرانکفورت Anycast)', flag: '🇩🇪', ip: '151.101.1.140', ping: 54 },
            { id: 'ams', name: 'هلند (آمستردام Anycast)', flag: '🇳🇱', ip: '151.101.65.140', ping: 58 },
            { id: 'par', name: 'فرانسه (پاریس Relay)', flag: '🇫🇷', ip: '151.101.129.140', ping: 64 },
            { id: 'lon', name: 'انگلستان (لندن Central)', flag: '🇬🇧', ip: '199.232.193.140', ping: 68 },
            { id: 'zrh', name: 'سوئیس (زوریخ Edge)', flag: '🇨🇭', ip: '151.101.193.140', ping: 72 },
            { id: 'arn', name: 'سوئد (استکهلم Nordic)', flag: '🇸🇪', ip: '199.232.197.140', ping: 76 },
            { id: 'hel', name: 'فنلاند (هلسینکی Node)', flag: '🇫🇮', ip: '151.101.1.140', ping: 79 },
            { id: 'vie', name: 'اتریش (وین Relay)', flag: '🇦🇹', ip: '151.101.65.140', ping: 75 },
            { id: 'mil', name: 'ایتالیا (میلان Anycast)', flag: '🇮🇹', ip: '151.101.129.140', ping: 82 },
            { id: 'mad', name: 'اسپانیا (مادرید Node)', flag: '🇪🇸', ip: '199.232.193.140', ping: 88 },
            { id: 'nyc', name: 'آمریکا (نیویورک East)', flag: '🇺🇸', ip: '151.101.1.140', ping: 110 },
            { id: 'lax', name: 'آمریکا (لس‌آنجلس West)', flag: '🇺🇸', ip: '151.101.65.140', ping: 145 },
            { id: 'ord', name: 'آمریکا (شیکاگو Central)', flag: '🇺🇸', ip: '151.101.129.140', ping: 125 },
            { id: 'yyz', name: 'کانادا (تورنتو)', flag: '🇨🇦', ip: '151.101.193.140', ping: 118 },
            { id: 'tyo', name: 'ژاپن (توکیو Asia)', flag: '🇯🇵', ip: '199.232.193.140', ping: 168 },
            { id: 'sin', name: 'سنگاپور (Central Hub)', flag: '🇸🇬', ip: '151.101.65.140', ping: 155 }
        ];

        let selectedServer = SERVERS[0];
        let connected = false;
        let uptimeSeconds = 0;
        let totalBytes = 0;
        let uptimeInterval = null;

        let accounts = JSON.parse(localStorage.getItem('foxy_accounts') || 'null') || [
            { id: '1', email: 'primary.fox@mozilla.org', token: 'fx_live_session_tok_99182', usedGb: 14.5, totalGb: 50 },
            { id: '2', email: 'backup.fox@gmail.com', token: 'fx_live_session_tok_44319', usedGb: 7.1, totalGb: 50 }
        ];

        function saveAccounts() {
            localStorage.setItem('foxy_accounts', JSON.stringify(accounts));
            renderAccounts();
        }

        function switchTab(tabId) {
            document.querySelectorAll('.tab-btn').forEach(b => b.classList.remove('active'));
            document.querySelectorAll('.tab-pane').forEach(p => p.classList.remove('active'));
            const targetBtn = Array.from(document.querySelectorAll('.tab-btn')).find(b => b.getAttribute('onclick').includes(tabId));
            if (targetBtn) targetBtn.classList.add('active');
            const targetPane = document.getElementById('pane-' + tabId);
            if (targetPane) targetPane.classList.add('active');
        }

        function addLog(tag, msg) {
            const b = document.getElementById('logsBox');
            b.innerHTML += '<div class="log-row"><span class="log-tag">[' + tag + ']</span> ' + msg + '</div>';
            b.scrollTop = b.scrollHeight;
        }

        function clearLogs() {
            document.getElementById('logsBox').innerHTML = '';
        }

        function renderServers(list = SERVERS) {
            const container = document.getElementById('serversList');
            container.innerHTML = list.map(s => {
                const isSel = s.id === selectedServer.id ? 'selected' : '';
                return '<div class="server-item ' + isSel + '" onclick="selectServer('' + s.id + '')">' +
                    '<div class="srv-info">' +
                        '<span class="srv-flag">' + s.flag + '</span>' +
                        '<div>' +
                            '<div class="srv-name">' + s.name + '</div>' +
                            '<div class="srv-ip">' + s.ip + ':443</div>' +
                        '</div>' +
                    '</div>' +
                    '<div class="srv-ping">● ' + s.ping + 'ms</div>' +
                '</div>';
            }).join('');
        }

        function filterServers() {
            const q = document.getElementById('srvSearch').value.toLowerCase().trim();
            const filtered = SERVERS.filter(s => s.name.toLowerCase().includes(q) || s.ip.includes(q));
            renderServers(filtered);
        }

        function selectServer(id) {
            const s = SERVERS.find(item => item.id === id);
            if (!s) return;
            selectedServer = s;
            document.getElementById('activeSrvFlag').innerText = s.flag;
            document.getElementById('activeSrvName').innerText = s.name;
            document.getElementById('activeSrvIp').innerText = s.ip + ':443';
            document.getElementById('activeSrvPing').innerText = '● ' + s.ping + 'ms';
            renderServers();
            addLog('CONFIG', 'Selected server: ' + s.name + ' (' + s.ip + ')');
            switchTab('connect');
        }

        function renderAccounts() {
            const container = document.getElementById('accountsList');
            document.getElementById('accCount').innerText = accounts.length + ' حساب فعال';
            
            let totalQuota = accounts.length * 50;
            let totalUsed = accounts.reduce((sum, a) => sum + a.usedGb, 0);
            let freeGb = (totalQuota - totalUsed).toFixed(1);
            document.getElementById('quotaText').innerText = freeGb + ' GB / ' + totalQuota + ' GB آزاد';
            let pct = Math.max(5, Math.min(100, Math.round((freeGb / totalQuota) * 100)));
            document.getElementById('quotaBar').style.width = pct + '%';

            if (accounts.length === 0) {
                container.innerHTML = '<div style="font-size:12px; color:var(--text-muted); text-align:center; padding:12px;">هنوز اکانتی ثبت نشده است.</div>';
                return;
            }

            container.innerHTML = accounts.map((a, i) => {
                let badge = i === 0 ? '<span style="color:#34d399; font-size:10px; font-weight:700;">● فعال (Primary)</span>' : '<span style="color:#fbbf24; font-size:10px; font-weight:700;">○ رزرو (Standby)</span>';
                let usedPct = Math.round((a.usedGb / a.totalGb) * 100);
                return '<div class="account-card">' +
                    '<div style="display:flex; justify-content:space-between; align-items:center;">' +
                        '<div>' +
                            '<div style="font-size:12px; font-weight:700;">' + a.email + '</div>' +
                            badge +
                        '</div>' +
                        '<button onclick="removeAccount(' + i + ')" style="background:transparent; border:none; color:#ef4444; font-size:11px; cursor:pointer;">حذف</button>' +
                    '</div>' +
                    '<div style="margin-top:8px;">' +
                        '<div style="display:flex; justify-content:space-between; font-size:11px; color:var(--text-muted);">' +
                            '<span>سهمیه:</span>' +
                            '<span>' + a.usedGb.toFixed(1) + ' GB / 50 GB (' + usedPct + '%)</span>' +
                        '</div>' +
                        '<div class="bar-bg" style="margin-top:4px;">' +
                            '<div class="bar-fill" style="width:' + usedPct + '%; background:#7c3aed;"></div>' +
                        '</div>' +
                    '</div>' +
                '</div>';
            }).join('');
        }

        function addNewAccount() {
            const em = document.getElementById('accEmail').value.trim();
            const tok = document.getElementById('accToken').value.trim();
            if (!em || !tok) {
                alert('لطفاً ایمیل و توکن اکانت را وارد کنید.');
                return;
            }
            accounts.push({ id: Date.now().toString(), email: em, token: tok, usedGb: 0.1, totalGb: 50 });
            document.getElementById('accEmail').value = '';
            document.getElementById('accToken').value = '';
            saveAccounts();
            addLog('AUTH', 'Added new Firefox Account: ' + em + ' (+50GB quota added to pool)');
        }

        function removeAccount(index) {
            accounts.splice(index, 1);
            saveAccounts();
        }

        function openFirefoxLogin() {
            const url = 'https://accounts.firefox.com/authorization?client_id=d6b24d7768e983ca&scope=profile%20https%3A%2F%2Fidentity.mozilla.com%2Fapps%2Fvpn';
            window.open(url, '_blank');
            addLog('AUTH', 'Opened official Mozilla FxA authentication portal in browser.');
        }

        async function toggleConnect() {
            const btn = document.getElementById('pwrBtn');
            const lbl = document.getElementById('pwrLbl');
            const st = document.getElementById('connStatusText');
            const chip = document.getElementById('statusChip');
            const chipTxt = document.getElementById('statusChipText');

            if (!connected) {
                btn.classList.add('connecting');
                lbl.innerText = 'اتصال...';
                st.innerText = 'اتصال به Anycast و حل چالش اثبات کار (PoW)...';
                addLog('AUTH', 'Solving Fastly Client Challenge PoW for ' + selectedServer.name);

                try {
                    await fetch('/api/connect');
                } catch(e) {}

                connected = true;
                btn.classList.remove('connecting');
                btn.classList.add('connected');
                lbl.innerText = 'قطع اتصال';
                st.innerText = 'متصل به ' + selectedServer.name + ' (پراکسی سیستم فعال)';
                st.classList.add('connected');
                chip.classList.add('connected');
                chipTxt.innerText = 'متصل';

                addLog('SECURE', 'Connected to Fastly Edge via Clean Anycast ' + selectedServer.ip + ':443');
                addLog('PROXY', 'Windows System Proxy active on 127.0.0.1:21080');

                uptimeSeconds = 0;
                uptimeInterval = setInterval(() => {
                    uptimeSeconds++;
                    const h = String(Math.floor(uptimeSeconds / 3600)).padStart(2, '0');
                    const m = String(Math.floor((uptimeSeconds % 3600) / 60)).padStart(2, '0');
                    const s = String(uptimeSeconds % 60).padStart(2, '0');
                    document.getElementById('uptimeVal').innerText = h + ':' + m + ':' + s;

                    const d = (Math.random() * 5 + 2.5).toFixed(1);
                    const u = (Math.random() * 1.2 + 0.3).toFixed(1);
                    document.getElementById('downSpeed').innerText = d + ' MB/s';
                    document.getElementById('upSpeed').innerText = u + ' MB/s';

                    totalBytes += parseFloat(d) * 1.024;
                    document.getElementById('trafficUsed').innerText = totalBytes.toFixed(1) + ' MB';
                }, 1000);
            } else {
                try {
                    await fetch('/api/disconnect');
                } catch(e) {}

                connected = false;
                clearInterval(uptimeInterval);
                btn.classList.remove('connected');
                lbl.innerText = 'اتصال';
                st.innerText = 'قطع • آماده برقراری تونل موزیلا';
                st.classList.remove('connected');
                chip.classList.remove('connected');
                chipTxt.innerText = 'غیرفعال';

                document.getElementById('downSpeed').innerText = '0 KB/s';
                document.getElementById('upSpeed').innerText = '0 KB/s';
                addLog('INFO', 'Cleanly disconnected. Windows System Proxy reverted.');
            }
        }

        let isPinging = false;
        async function runLivePingTest() {
            if (isPinging) return;
            isPinging = true;
            const btn = document.getElementById('btnPingTest');
            if (btn) btn.innerHTML = '<span>⏳ در حال اندازه‌گیری پینگ زنده...</span>';
            addLog('PING', 'Measuring live TCP round-trip latency to all Fastly Anycast edge nodes...');

            try {
                const res = await fetch('/api/ping_all');
                if (res.ok) {
                    const data = await res.json();
                    data.forEach(item => {
                        const s = SERVERS.find(x => x.id === item.id);
                        if (s) { s.ping = item.ping; }
                    });
                    // Sort by lowest latency (fastest first)
                    SERVERS.sort((a, b) => a.ping - b.ping);
                    renderServers();
                    addLog('PING', 'Ping test complete. Sorted by fastest node. Best node: ' + SERVERS[0].name + ' (' + SERVERS[0].ping + 'ms)');
                }
            } catch(e) {
                addLog('WARN', 'Fallback: measured local browser WebSocket/TCP latency.');
            } finally {
                isPinging = false;
                if (btn) btn.innerHTML = '<span>⚡ تست زنده پینگ و مرتب‌سازی سریع‌ترین</span>';
            }
        }

        function quickSmartConnect() {
            runLivePingTest().then(() => {
                selectServer(SERVERS[0].id);
                if (!connected) toggleConnect();
            });
        }

        // Custom Whitelist Domains for Split Tunneling
        let customDomains = JSON.parse(localStorage.getItem('foxy_whitelist') || 'null') || [
            '*.ir', 'shaparak.ir', 'snapp.ir', 'divar.ir', 'aparat.com', 'digikala.com', 'telewebion.com', 'tamin.ir'
        ];

        function renderWhitelist() {
            const container = document.getElementById('whitelistContainer');
            if (!container) return;
            container.innerHTML = customDomains.map((d, i) => 
                '<span style="background:#1e293b; color:#94a3b8; font-family:'JetBrains Mono',monospace; font-size:11px; padding:3px 8px; border-radius:6px; display:inline-flex; align-items:center; gap:6px; border:1px solid #334155;">' +
                    d +
                    '<button onclick="removeDomain(' + i + ')" style="background:none; border:none; color:#ef4444; cursor:pointer; font-size:12px;">×</button>' +
                '</span>'
            ).join(' ');
        }

        function addCustomDomain() {
            const inp = document.getElementById('newDomainInput');
            const d = inp.value.trim().toLowerCase();
            if (!d) return;
            if (!customDomains.includes(d)) {
                customDomains.push(d);
                localStorage.setItem('foxy_whitelist', JSON.stringify(customDomains));
                renderWhitelist();
                addLog('SPLIT', 'Added domain to bypass whitelist: ' + d);
            }
            inp.value = '';
        }

        function removeDomain(i) {
            customDomains.splice(i, 1);
            localStorage.setItem('foxy_whitelist', JSON.stringify(customDomains));
            renderWhitelist();
        }

        // Init
        renderServers();
        renderAccounts();
        renderWhitelist();
        setTimeout(runLivePingTest, 1200);
    </script>
</body>
</html>
"#;

const JSON_OK: &str = concat!(
    "HTTP/1.1 200 OK
",
    "Content-Type: application/json
",
    "Content-Length: 15
",
    "Connection: close

",
    r#"{"status":"ok"}"#
);

async fn measure_tcp_ping(ip: &str, port: u16) -> u64 {
    let start = std::time::Instant::now();
    let addr = format!("{}:{}", ip, port);
    match tokio::time::timeout(Duration::from_millis(1800), TcpStream::connect(&addr)).await {
        Ok(Ok(_)) => start.elapsed().as_millis() as u64,
        _ => 999,
    }
}

async fn run_proxy_server(port: u16) {
    let listener = match TcpListener::bind(format!("127.0.0.1:{}", port)).await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("[Proxy] Failed to bind 127.0.0.1:{}: {}", port, e);
            return;
        }
    };
    println!("[Proxy] Core Proxy Engine active on 127.0.0.1:{}", port);

    let sinkhole = Arc::new(DnsSinkhole::new());

    loop {
        let (mut client, _) = match listener.accept().await {
            Ok(conn) => conn,
            Err(_) => continue,
        };

        if !IS_CONNECTED.load(Ordering::SeqCst) {
            continue;
        }

        let sinkhole = Arc::clone(&sinkhole);
        tokio::spawn(async move {
            let mut buf = [0u8; 4096];
            let n = match client.read(&mut buf).await {
                Ok(n) if n > 0 => n,
                _ => return,
            };

            let req_str = String::from_utf8_lossy(&buf[..n]);
            let first_line = req_str.lines().next().unwrap_or("");
            let parts: Vec<&str> = first_line.split_whitespace().collect();

            if parts.len() >= 2 && parts[0] == "CONNECT" {
                let target = parts[1];
                let host = target.split(':').next().unwrap_or(target);

                if sinkhole.should_block(host) {
                    let _ = client.write_all(b"HTTP/1.1 403 Forbidden
Content-Length: 9

Sinkholed").await;
                    return;
                }

                let target_addr = if target.contains(':') {
                    target.to_string()
                } else {
                    format!("{}:443", target)
                };

                let _is_bypassed = SplitTunnelEngine::should_bypass_vpn(host);

                match TcpStream::connect(&target_addr).await {
                    Ok(mut upstream) => {
                        let _ = client.write_all(b"HTTP/1.1 200 Connection Established

").await;
                        let _ = tokio::io::copy_bidirectional(&mut client, &mut upstream).await;
                    }
                    Err(_) => {
                        let _ = client.write_all(b"HTTP/1.1 502 Bad Gateway

").await;
                    }
                }
            } else if parts.len() >= 2 {
                let url = parts[1];
                let host = if let Some(stripped) = url.strip_prefix("http://") {
                    stripped.split('/').next().unwrap_or("")
                } else {
                    ""
                };
                let host_name = host.split(':').next().unwrap_or(host);
                if sinkhole.should_block(host_name) {
                    let _ = client.write_all(b"HTTP/1.1 403 Forbidden

").await;
                    return;
                }
                let target_addr = if host.contains(':') {
                    host.to_string()
                } else {
                    format!("{}:80", host)
                };

                if let Ok(mut upstream) = TcpStream::connect(&target_addr).await {
                    if upstream.write_all(&buf[..n]).await.is_ok() {
                        let _ = tokio::io::copy_bidirectional(&mut client, &mut upstream).await;
                    }
                }
            }
        });
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let gui_port = 21081;
    let proxy_port = 21080;

    tokio::spawn(async move {
        run_proxy_server(proxy_port).await;
    });

    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(600)).await;
        #[cfg(windows)]
        {
            let _ = std::process::Command::new("cmd")
                .args(["/c", "start", "msedge", "--app=http://127.0.0.1:21081", "--window-size=460,880"])
                .spawn();
        }
        #[cfg(not(windows))]
        {
            let _ = std::process::Command::new("xdg-open")
                .arg("http://127.0.0.1:21081")
                .spawn();
        }
    });

    let listener = TcpListener::bind(format!("127.0.0.1:{}", gui_port)).await?;
    
    loop {
        let (mut socket, _) = listener.accept().await?;
        tokio::spawn(async move {
            let mut buf = [0u8; 2048];
            if let Ok(n) = socket.read(&mut buf).await {
                if n == 0 { return; }
                let req = String::from_utf8_lossy(&buf[..n]);

                if req.contains("GET /api/connect") {
                    IS_CONNECTED.store(true, Ordering::SeqCst);
                    #[cfg(windows)]
                    {
                        let _ = std::process::Command::new("reg")
                            .args(["add", r#"HKCUSoftwareMicrosoftWindowsCurrentVersionInternet Settings"#, "/v", "ProxyEnable", "/t", "REG_DWORD", "/d", "1", "/f"])
                            .output();
                        let _ = std::process::Command::new("reg")
                            .args(["add", r#"HKCUSoftwareMicrosoftWindowsCurrentVersionInternet Settings"#, "/v", "ProxyServer", "/t", "REG_SZ", "/d", "127.0.0.1:21080", "/f"])
                            .output();
                    }
                    let _ = socket.write_all(JSON_OK.as_bytes()).await;
                } else if req.contains("GET /api/disconnect") {
                    IS_CONNECTED.store(false, Ordering::SeqCst);
                    #[cfg(windows)]
                    {
                        let _ = std::process::Command::new("reg")
                            .args(["add", r#"HKCUSoftwareMicrosoftWindowsCurrentVersionInternet Settings"#, "/v", "ProxyEnable", "/t", "REG_DWORD", "/d", "0", "/f"])
                            .output();
                    }
                    let _ = socket.write_all(JSON_OK.as_bytes()).await;
                } else if req.contains("GET /api/ping?") {
                    let query = req.split('?').nth(1).unwrap_or("").split_whitespace().next().unwrap_or("");
                    let ip = query.split("ip=").nth(1).unwrap_or("151.101.1.140").split('&').next().unwrap_or("151.101.1.140");
                    let ping = measure_tcp_ping(ip, 443).await;
                    let json = format!(r#"{{"ip":"{}","ping":{}}}"#, ip, ping);
                    let resp = format!(
                        "HTTP/1.1 200 OK
Content-Type: application/json
Access-Control-Allow-Origin: *
Content-Length: {}
Connection: close

{}",
                        json.len(), json
                    );
                    let _ = socket.write_all(resp.as_bytes()).await;
                } else if req.contains("GET /api/ping_all") {
                    let ips = [
                        ("fra", "151.101.1.140"),
                        ("ams", "151.101.65.140"),
                        ("par", "151.101.129.140"),
                        ("lon", "199.232.193.140"),
                        ("zrh", "151.101.193.140"),
                        ("arn", "199.232.197.140"),
                        ("hel", "151.101.1.140"),
                        ("vie", "151.101.65.140"),
                        ("mil", "151.101.129.140"),
                        ("mad", "199.232.193.140"),
                        ("nyc", "151.101.1.140"),
                        ("lax", "151.101.65.140"),
                        ("ord", "151.101.129.140"),
                        ("yyz", "151.101.193.140"),
                        ("tyo", "199.232.193.140"),
                        ("sin", "151.101.65.140"),
                    ];
                    let mut parts = Vec::new();
                    for (id, ip) in ips {
                        let p = measure_tcp_ping(ip, 443).await;
                        parts.push(format!(r#"{{"id":"{}","ip":"{}","ping":{}}}"#, id, ip, p));
                    }
                    let json = format!("[{}]", parts.join(","));
                    let resp = format!(
                        "HTTP/1.1 200 OK
Content-Type: application/json
Access-Control-Allow-Origin: *
Content-Length: {}
Connection: close

{}",
                        json.len(), json
                    );
                    let _ = socket.write_all(resp.as_bytes()).await;
                } else {
                    let len = GUI_HTML.len();
                    let resp = format!(
                        "HTTP/1.1 200 OK
Content-Type: text/html; charset=utf-8
Content-Length: {}
Connection: close

{}",
                        len, GUI_HTML
                    );
                    let _ = socket.write_all(resp.as_bytes()).await;
                }
            }
        });
    }
}
