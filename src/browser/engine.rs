use anyhow::{Context, Result};
use crate::browser::Navigation;
use std::sync::{Arc, Mutex};

pub struct Browser {
    navigation: Navigation,
}

impl Browser {
    pub fn new() -> Result<Self> {
        Ok(Self {
            navigation: Navigation::new(),
        })
    }

    pub fn run(&mut self) -> Result<()> {
        // EventLoop must be created on the main thread (macOS requirement)
        let navigation = Arc::new(Mutex::new(std::mem::take(&mut self.navigation)));
        Self::run_event_loop(navigation)
    }

    fn run_event_loop(navigation: Arc<Mutex<Navigation>>) -> Result<()> {
        use wry::{
            application::{
                event::{Event, StartCause, WindowEvent},
                event_loop::{ControlFlow, EventLoop},
                window::WindowBuilder,
            },
            webview::WebViewBuilder,
        };

        let event_loop = EventLoop::new();
        
        let window = WindowBuilder::new()
            .with_title("SyncFlo Browser")
            .with_inner_size(wry::application::dpi::LogicalSize::new(1280.0, 800.0))
            .build(&event_loop)
            .context("Failed to create window")?;

        let nav_clone = navigation.clone();
        
        // Create two windows: nav bar (top, 56px) and main content
        use wry::application::dpi::{LogicalPosition, LogicalSize};
        use std::rc::Rc;
        use std::cell::RefCell;
        
        // Get window position and size for alignment
        let window_pos = window.outer_position().unwrap_or(LogicalPosition::new(100.0, 100.0));
        let window_size = window.outer_size();
        let nav_height = 56.0;
        
        // Adjust main window to account for nav bar
        window.set_inner_size(LogicalSize::new(window_size.width as f64, (window_size.height as f64) - nav_height));
        window.set_position(LogicalPosition::new(window_pos.x, window_pos.y + nav_height));
        
        // Nav bar window (top, 56px, no decorations, always on top, clickable)
        let nav_window = WindowBuilder::new()
            .with_title("")
            .with_decorations(false) // No title bar, no borders
            .with_inner_size(LogicalSize::new(window_size.width as f64, nav_height))
            .with_position(LogicalPosition::new(window_pos.x, window_pos.y))
            .with_always_on_top(true) // Keep nav always on top
            .build(&event_loop)
            .context("Failed to create nav window")?;
        
        // Create content webview in original window
        let content_webview = WebViewBuilder::new(window)?
            .with_url("about:blank")?
            .with_devtools(true)
            .build()?;
        let content_wv_rc = Rc::new(RefCell::new(content_webview));
        
        // Create nav webview with IPC handler
        let nav_url = Self::local_nav_file_url()?;
        let content_for_ipc = content_wv_rc.clone();
        let _nav_webview = WebViewBuilder::new(nav_window)?
            .with_url(&nav_url)?
            .with_ipc_handler(move |_, msg| {
                let text = msg;
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) {
                    let op = v.get("op").and_then(|x| x.as_str()).unwrap_or("");
                    match op {
                        "back" => { let _ = content_for_ipc.borrow().evaluate_script("window.history.back()"); },
                        "forward" => { let _ = content_for_ipc.borrow().evaluate_script("window.history.forward()"); },
                        "refresh" => { let _ = content_for_ipc.borrow().evaluate_script("window.location.reload()"); },
                        "home" => {
                            if let Ok(url) = Self::local_home_file_url() { 
                                let _ = content_for_ipc.borrow().load_url(&url); 
                            }
                        },
                        "navigate" => {
                            if let Some(u) = v.get("payload").and_then(|p| p.get("url")).and_then(|x| x.as_str()) {
                                let target = if u.starts_with("http://") || u.starts_with("https://") { 
                                    u.to_string() 
                                } else if u.contains('.') && !u.contains(' ') { 
                                    format!("https://{}", u) 
                                } else { 
                                    format!("https://www.google.com/search?q={}", urlencoding::encode(u)) 
                                };
                                let _ = content_for_ipc.borrow().load_url(&target);
                            }
                        },
                        _ => {}
                    }
                }
            })
            .build()?;
        
        // Load home page initially
        if let Ok(u) = Self::local_home_file_url() { 
            let _ = content_wv_rc.borrow().load_url(&u); 
        }
        
        // Store window IDs for synchronization
        let nav_window_id = nav_window.id();
        let content_window_id = window.id();

        let nav_for_keys = navigation.clone();

        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Wait;

            match event {
                Event::NewEvents(StartCause::Init) => {
                    log::info!("SyncFlo Browser initialized");
                    // Initialize navigation with start page
                    if let Ok(mut nav) = nav_for_keys.lock() {
                        let _ = nav.navigate("data:text/html,start".to_string());
                    }
                }
                Event::WindowEvent {
                    window_id,
                    event: WindowEvent::Moved(pos),
                } => {
                    // Sync nav window position when content window moves
                    if window_id == content_window_id {
                        if let Ok(w) = nav_window.request_redraw() {
                            // Note: Direct window manipulation may be limited in wry 0.24
                            // The nav window should stay on top due to with_always_on_top(true)
                        }
                    }
                }
                Event::WindowEvent {
                    window_id,
                    event: WindowEvent::CloseRequested,
                } => {
                    // Close both windows when one is closed
                    if window_id == content_window_id || window_id == nav_window_id {
                        *control_flow = ControlFlow::Exit;
                    }
                }
                _ => {}
            }
        });
        
        // This line is unreachable because event_loop.run() blocks until exit
        // but we keep it for clarity and potential future changes
        #[allow(unreachable_code)]
        Ok(())
    }

    fn local_app_file_url() -> Result<String> {
        use std::path::{Path, PathBuf};
        // During development, assets/home.html is relative to project root.
        // We resolve it from current working directory.
        let cwd = std::env::current_dir().context("Failed to get current_dir")?;
        let path: PathBuf = cwd.join("assets").join("app.html");
        if !path.exists() {
            return Err(anyhow::anyhow!(
                "Home file not found: {}",
                path.to_string_lossy()
            ));
        }
        let url = format!("file://{}", path.to_string_lossy());
        Ok(url)
    }

    fn local_nav_file_url() -> Result<String> {
        use std::path::PathBuf;
        let cwd = std::env::current_dir().context("Failed to get current_dir")?;
        let path: PathBuf = cwd.join("assets").join("nav.html");
        if !path.exists() {
            return Err(anyhow::anyhow!(
                "Nav file not found: {}",
                path.to_string_lossy()
            ));
        }
        Ok(format!("file://{}", path.to_string_lossy()))
    }

    fn local_home_file_url() -> Result<String> {
        use std::path::PathBuf;
        let cwd = std::env::current_dir().context("Failed to get current_dir")?;
        let path: PathBuf = cwd.join("assets").join("home.html");
        if !path.exists() {
            return Err(anyhow::anyhow!(
                "Home file not found: {}",
                path.to_string_lossy()
            ));
        }
        Ok(format!("file://{}", path.to_string_lossy()))
    }

    fn build_start_page_html() -> String {
        // Start page with navigation bar and centered search box
        r#"<!DOCTYPE html>
<html lang="ko">
<head>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1" />
  <title>SyncFlo Start</title>
  <style>
    * { box-sizing: border-box; }
    html, body { height: 100%; margin: 0; font-family: -apple-system, BlinkMacSystemFont, Segoe UI, Roboto, Helvetica, Arial, sans-serif; background: #121212; color: #e6e6e6; }
    
    /* Navigation Bar */
    .navbar { position: fixed; top: 0; left: 0; right: 0; height: 56px; background: #1e1e1e; border-bottom: 1px solid #2a2a2a; display: flex; align-items: center; gap: 8px; padding: 0 12px; z-index: 1000; }
    .nav-btn { width: 36px; height: 36px; border: none; background: #2a2a2a; color: #e6e6e6; border-radius: 6px; cursor: pointer; display: flex; align-items: center; justify-content: center; font-size: 16px; transition: background 0.2s; }
    .nav-btn:hover { background: #3a3a3a; }
    .nav-btn:active { background: #1a1a1a; }
    .nav-btn:disabled { opacity: 0.4; cursor: not-allowed; }
    .address-bar { flex: 1; height: 36px; padding: 0 12px; border-radius: 6px; border: 1px solid #2a2a2a; background: #1b1b1b; color: #e6e6e6; outline: none; font-size: 14px; }
    .address-bar:focus { border-color: #3a83f7; box-shadow: 0 0 0 2px rgba(58,131,247,0.25); }
    .refresh-btn { width: 36px; height: 36px; }
    
    /* Content Area */
    .content { padding-top: 56px; min-height: 100%; display: flex; align-items: center; justify-content: center; }
    .wrap { width: min(720px, 92vw); text-align: center; }
    h1 { font-size: 28px; font-weight: 700; margin-bottom: 20px; color: #fafafa; }
    form { display: flex; gap: 8px; }
    input[type=text] { flex: 1; height: 48px; padding: 0 16px; border-radius: 12px; border: 1px solid #2a2a2a; background: #1b1b1b; color: #e6e6e6; outline: none; font-size: 16px; }
    input[type=text]:focus { border-color: #3a83f7; box-shadow: 0 0 0 3px rgba(58,131,247,0.25); }
    button { height: 48px; padding: 0 18px; border-radius: 12px; border: 0; background: #3a83f7; color: white; font-size: 16px; cursor: pointer; }
    button:hover { background: #2f73e1; }
    .hint { margin-top: 12px; color: #a7a7a7; font-size: 13px; }
    
    /* Start page only; no global injection on external pages */
  </style>
  <script>
    (function() {
      function isUrl(str) {
        try {
          var url = new URL(str);
          return url.protocol === 'http:' || url.protocol === 'https:';
        } catch(e) {
          if (str.indexOf('.') > 0 && (str.indexOf(' ') === -1)) {
            return true;
          }
          return false;
        }
      }

      function handleSearch(query) {
        query = query.trim();
        if (!query) return;
        
        var url;
        if (isUrl(query)) {
          if (!query.startsWith('http://') && !query.startsWith('https://')) {
            url = 'https://' + query;
          } else {
            url = query;
          }
        } else {
          url = 'https://www.google.com/search?q=' + encodeURIComponent(query);
        }
        
        window.location.href = url;
      }

      function onSubmit(e) {
        e.preventDefault();
        var input = document.getElementById('q');
        handleSearch(input.value);
        return false;
      }

      function onButtonClick() {
        var input = document.getElementById('q');
        handleSearch(input.value);
      }


      // Navigation bar handlers
      function initNavBar() {
        var backBtn = document.getElementById('navBack');
        var homeBtn = document.getElementById('navHome');
        var forwardBtn = document.getElementById('navForward');
        var refreshBtn = document.getElementById('navRefresh');
        var addressBar = document.getElementById('addressBar');
        
        if (backBtn) {
          backBtn.addEventListener('click', function() {
            var before = window.location.href;
            if (window.history.length > 1) {
              window.history.back();
              setTimeout(function(){
                if (window.location.href === before) {
                  // fallback to home if no back navigation happened
                  window.location.href = window.SYNCFLO_HOME || '#';
                }
              }, 400);
            } else {
              window.location.href = window.SYNCFLO_HOME || '#';
            }
          });
        }

        if (homeBtn) {
          homeBtn.addEventListener('click', function(){
            window.location.href = window.SYNCFLO_HOME || '#';
          });
        }
        
        if (forwardBtn) {
          forwardBtn.addEventListener('click', function() {
            window.history.forward();
          });
        }
        
        if (refreshBtn) {
          refreshBtn.addEventListener('click', function() {
            window.location.reload();
          });
        }
        
        if (addressBar) {
          addressBar.addEventListener('keydown', function(e) {
            if (e.key === 'Enter') {
              e.preventDefault();
              handleSearch(addressBar.value);
            }
          });
          
          // Update address bar on navigation
          window.addEventListener('popstate', function() {
            addressBar.value = window.location.href;
          });
          
          // Update on page load
          addressBar.value = window.location.href;
        }
        
        // Update navigation buttons state
        function updateNavButtons() {
          if (backBtn) {
            backBtn.disabled = window.history.length <= 1;
          }
          if (forwardBtn) {
            // Forward state is harder to detect, keep it enabled
            forwardBtn.disabled = false;
          }
        }
        
        updateNavButtons();
        setInterval(updateNavButtons, 500);
      }

      window.addEventListener('DOMContentLoaded', function() {
        document.body.classList.add('start-page');
        
        var form = document.getElementById('searchForm');
        var input = document.getElementById('q');
        var button = document.getElementById('searchBtn');
        
        if (form) form.addEventListener('submit', onSubmit);
        if (button) button.addEventListener('click', onButtonClick);
        
        // Note: Global keyboard shortcuts are disabled to avoid WebKit crashes
        
        // Initialize navigation bar
        initNavBar();
        
        if (input) input.focus();
        
        // Update address bar when page changes
        var observer = new MutationObserver(function() {
          var addressBar = document.getElementById('addressBar');
          if (addressBar) {
            addressBar.value = window.location.href;
          }
        });
        
        observer.observe(document.body, { childList: true, subtree: true });
      });
      
      // Update address bar on navigation
      window.addEventListener('load', function() {
        document.body.classList.remove('start-page');
        var addressBar = document.getElementById('addressBar');
        if (addressBar) {
          addressBar.value = window.location.href;
        }
        
        // Inject navigation bar on all pages (including external pages)
        if (!document.getElementById('syncflo-navbar')) {
          injectNavBar();
        }
      });
      
      // Inject navigation bar function - runs on all pages
      function injectNavBar() {
        // Only inject if navbar doesn't exist
        if (document.getElementById('syncflo-navbar')) return;
        
        // Navigation bar HTML
        var navbarHTML = `
          <div id="syncflo-navbar" style="position: fixed; top: 0; left: 0; right: 0; height: 56px; background: #1e1e1e; border-bottom: 1px solid #2a2a2a; display: flex; align-items: center; gap: 8px; padding: 0 12px; z-index: 999999; font-family: -apple-system, BlinkMacSystemFont, Segoe UI, Roboto, Helvetica, Arial, sans-serif;">
            <button id="syncflo-navBack" style="width: 36px; height: 36px; border: none; background: #2a2a2a; color: #e6e6e6; border-radius: 6px; cursor: pointer; display: flex; align-items: center; justify-content: center; font-size: 16px; transition: background 0.2s;" title="뒤로가기 (Cmd+[)">←</button>
            <button id="syncflo-navForward" style="width: 36px; height: 36px; border: none; background: #2a2a2a; color: #e6e6e6; border-radius: 6px; cursor: pointer; display: flex; align-items: center; justify-content: center; font-size: 16px; transition: background 0.2s;" title="앞으로가기 (Cmd+])">→</button>
            <button id="syncflo-navRefresh" style="width: 36px; height: 36px; border: none; background: #2a2a2a; color: #e6e6e6; border-radius: 6px; cursor: pointer; display: flex; align-items: center; justify-content: center; font-size: 16px; transition: background 0.2s;" title="새로고침 (Cmd+R)">⟳</button>
            <input id="syncflo-addressBar" type="text" style="flex: 1; height: 36px; padding: 0 12px; border-radius: 6px; border: 1px solid #2a2a2a; background: #1b1b1b; color: #e6e6e6; outline: none; font-size: 14px;" placeholder="주소 또는 검색어 입력" />
          </div>
        `;
        
        // Inject navbar at the beginning of body
        if (document.body) {
          document.body.insertAdjacentHTML('afterbegin', navbarHTML);
          
          // Add padding to body to prevent content from being hidden under navbar
          if (!document.body.style.paddingTop) {
            document.body.style.paddingTop = '56px';
          }
          
          // Initialize navigation handlers
          var backBtn = document.getElementById('syncflo-navBack');
          var forwardBtn = document.getElementById('syncflo-navForward');
          var refreshBtn = document.getElementById('syncflo-navRefresh');
          var addressBar = document.getElementById('syncflo-addressBar');
          
          if (backBtn) {
            backBtn.addEventListener('click', function() {
              window.history.back();
            });
            backBtn.addEventListener('mouseenter', function() { this.style.background = '#3a3a3a'; });
            backBtn.addEventListener('mouseleave', function() { this.style.background = '#2a2a2a'; });
          }
          
          if (forwardBtn) {
            forwardBtn.addEventListener('click', function() {
              window.history.forward();
            });
            forwardBtn.addEventListener('mouseenter', function() { this.style.background = '#3a3a3a'; });
            forwardBtn.addEventListener('mouseleave', function() { this.style.background = '#2a2a2a'; });
          }
          
          if (refreshBtn) {
            refreshBtn.addEventListener('click', function() {
              window.location.reload();
            });
            refreshBtn.addEventListener('mouseenter', function() { this.style.background = '#3a3a3a'; });
            refreshBtn.addEventListener('mouseleave', function() { this.style.background = '#2a2a2a'; });
          }
          
          if (addressBar) {
            addressBar.value = window.location.href;
            
            addressBar.addEventListener('keydown', function(e) {
              if (e.key === 'Enter') {
                e.preventDefault();
                var url = addressBar.value.trim();
                if (!url) return;
                
                // Check if URL or search query
                var isUrl = false;
                try {
                  var testUrl = new URL(url);
                  isUrl = testUrl.protocol === 'http:' || testUrl.protocol === 'https:';
                } catch(e) {
                  if (url.indexOf('.') > 0 && url.indexOf(' ') === -1) {
                    isUrl = true;
                    url = 'https://' + url;
                  }
                }
                
                if (isUrl) {
                  window.location.href = url;
                } else {
                  window.location.href = 'https://www.google.com/search?q=' + encodeURIComponent(url);
                }
              }
            });
            
            // Update address bar on navigation
            window.addEventListener('popstate', function() {
              if (addressBar) addressBar.value = window.location.href;
            });
          }
          
          // Update address bar periodically
          setInterval(function() {
            if (addressBar && addressBar !== document.activeElement) {
              addressBar.value = window.location.href;
            }
            if (backBtn) {
              backBtn.disabled = window.history.length <= 1;
            }
          }, 500);
        }
      }
      
      // Inject navbar on DOMContentLoaded as well (for pages that load quickly)
      if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', injectNavBar);
      } else {
        injectNavBar();
      }
    })();
  </script>
</head>
<body>
  <!-- Navigation Bar -->
  <div class="navbar">
    <button id="navBack" class="nav-btn" title="뒤로가기 (Cmd+[)">←</button>
    <button id="navHome" class="nav-btn" title="홈으로">⌂</button>
    <button id="navForward" class="nav-btn" title="앞으로가기 (Cmd+])">→</button>
    <button id="navRefresh" class="nav-btn refresh-btn" title="새로고침 (Cmd+R)">⟳</button>
    <input id="addressBar" type="text" class="address-bar" placeholder="주소 또는 검색어 입력" />
  </div>
  
  <!-- Content Area -->
  <div class="content">
    <div class="wrap">
      <div class="box">
        <h1>SyncFlo Browser</h1>
        <form id="searchForm" autocomplete="on">
          <input id="q" type="text" placeholder="브라우저 검색" />
          <button id="searchBtn" type="button">검색</button>
        </form>
        <div class="hint">Enter로 검색 · URL을 입력하면 해당 사이트로 이동합니다</div>
      </div>
    </div>
  </div>
</body>
</html>"#.to_string()
    }

    // No converter needed when using with_html()

    // Build data URL for the start page so JS can navigate "home"
    fn build_start_page_data_url() -> String {
        let html = Self::build_start_page_html();
        let encoded = base64::encode(html);
        format!("data:text/html;base64,{}", encoded)
    }

    // Global script injected on every page load to render a minimal navigation bar
    fn build_global_nav_script(home_data_url: &str) -> String {
        let template = r#"
(function() {
  try {
    function ensureNavbar() {
      if (document.getElementById('syncflo-navbar')) return;

      var navbar = document.createElement('div');
      navbar.id = 'syncflo-navbar';
      navbar.style.cssText = 'position:fixed;top:0;left:0;right:0;height:44px;background:rgba(30,30,30,0.95);backdrop-filter:saturate(150%) blur(6px);border-bottom:1px solid #2a2a2a;display:flex;align-items:center;gap:8px;padding:0 8px;z-index:2147483647;pointer-events:auto;box-shadow:0 2px 8px rgba(0,0,0,0.35);font-family:-apple-system,BlinkMacSystemFont,Segoe UI,Roboto,Helvetica,Arial,sans-serif;';

      function mkBtn(id, text, title) {
        var b = document.createElement('button');
        b.id = id; b.textContent = text; b.title = title;
        b.style.cssText = 'width:32px;height:32px;border:none;background:#2a2a2a;color:#e6e6e6;border-radius:6px;cursor:pointer;display:flex;align-items:center;justify-content:center;font-size:14px;';
        b.addEventListener('mouseenter', function(){ this.style.background = '#3a3a3a'; });
        b.addEventListener('mouseleave', function(){ this.style.background = '#2a2a2a'; });
        return b;
      }

      var back = mkBtn('syncflo-navBack', '←', '뒤로가기');
      var home = mkBtn('syncflo-navHome', '⌂', '홈으로 (Cmd+H)');
      var fwd  = mkBtn('syncflo-navForward', '→', '앞으로가기');
      var ref  = mkBtn('syncflo-navRefresh', '⟳', '새로고침');
      var devtools = mkBtn('syncflo-devTools', '⚙', '개발자 도구 (F12)');
      var addr = document.createElement('input');
      addr.id = 'syncflo-addressBar';
      addr.placeholder = '주소 또는 검색어 입력';
      addr.style.cssText = 'flex:1;height:32px;padding:0 10px;border-radius:6px;border:1px solid #2a2a2a;background:#1b1b1b;color:#e6e6e6;outline:none;font-size:13px;';

      navbar.appendChild(back); navbar.appendChild(home); navbar.appendChild(fwd); navbar.appendChild(ref); navbar.appendChild(devtools); navbar.appendChild(addr);

      document.documentElement.appendChild(navbar);
      var body = document.body || document.documentElement;
      if (body) {
        // Reserve space so page elements aren't interactable behind the navbar
        var current = parseInt((body.style.paddingTop||'0').replace('px','')) || 0;
        if (current < 60) body.style.paddingTop = '60px';
      }

      function handleEnter(url) {
        url = (url || '').trim(); if (!url) return;
        var isUrl = false;
        try { var u = new URL(url); isUrl = (u.protocol === 'http:' || u.protocol === 'https:'); } catch(e) {
          if (url.indexOf('.')>0 && url.indexOf(' ')===-1) { isUrl = true; url = 'https://' + url; }
        }
        window.location.href = isUrl ? url : ('https://www.google.com/search?q=' + encodeURIComponent(url));
      }

      var homeUrl = window.SYNCFLO_HOME || '{HOME}';
      
      // Enhanced back button - go to home if can't go back
      back.onclick = function(){
        try {
          // Try history.back() first
          if (history.length > 1 && document.referrer) {
            var before = location.href;
            history.back();
            // Check after a delay if we actually navigated back
            setTimeout(function(){ 
              if (location.href === before || location.href === document.referrer) {
                // Didn't navigate, go to home instead
                window.location.replace(homeUrl);
              }
            }, 100);
          } else {
            // No history, go directly to home
            window.location.replace(homeUrl);
          }
        } catch(e) {
          // Fallback to home on any error
          window.location.replace(homeUrl);
        }
      };
      
      home.onclick = function(){ window.location.replace(homeUrl); };
      fwd.onclick  = function(){ history.forward(); };
      ref.onclick  = function(){ location.reload(); };
      devtools.onclick = function(){ console.log('DevTools: macOS에서는 Cmd+Option+I를 사용하세요.'); alert('DevTools: macOS에서는 Cmd+Option+I를 사용하세요.'); };
      addr.onkeydown = function(e){ if (e.key === 'Enter') { e.preventDefault(); handleEnter(addr.value); } };

      function syncAddr(){ if (document.activeElement !== addr) addr.value = location.href; }
      syncAddr(); setInterval(syncAddr, 700);
      
      // Add keyboard shortcut for home (Cmd+H or Ctrl+H)
      document.addEventListener('keydown', function(e) {
        if ((e.metaKey || e.ctrlKey) && e.key === 'h') {
          e.preventDefault();
          window.location.replace(homeUrl);
        }
      });
    }

    if (document.readyState === 'loading') {
      document.addEventListener('DOMContentLoaded', ensureNavbar, { once:true });
    } else {
      ensureNavbar();
    }
    window.addEventListener('load', ensureNavbar);
    // expose home URL for all pages
    try { window.SYNCFLO_HOME = window.SYNCFLO_HOME || '{HOME}'; } catch(_) {}
  } catch (e) { /* ignore */ }
})();
"#;
        template.replace("{HOME}", home_data_url)
    }
}
