/* app.js */

// Global Initialization
document.addEventListener("DOMContentLoaded", () => {
  initNavigation();
  initCopyAction();
  initKeySearch();
});

// Mobile Navigation Hamburg Menu Toggle
function initNavigation() {
  const toggle = document.getElementById("menu-toggle");
  const links = document.getElementById("nav-links");

  if (toggle && links) {
    toggle.addEventListener("click", () => {
      links.classList.toggle("active");
      toggle.classList.toggle("active");
    });
  }
}

// Installation Command Copy Handler
function initCopyAction() {
  const copyBtn = document.getElementById("copy-btn");
  const codeBlock = document.getElementById("code-block");

  if (copyBtn && codeBlock) {
    copyBtn.addEventListener("click", () => {
      const rawText = codeBlock.innerText;
      navigator.clipboard
        .writeText(rawText)
        .then(() => {
          copyBtn.innerText = "Copied!";
          copyBtn.style.borderColor = "var(--terminal-green)";
          copyBtn.style.color = "var(--terminal-green)";

          setTimeout(() => {
            copyBtn.innerText = "Copy";
            copyBtn.style.borderColor = "var(--border-color)";
            copyBtn.style.color = "var(--text-color)";
          }, 2000);
        })
        .catch((err) => {
          console.error("Could not copy terminal text: ", err);
        });
    });
  }
}

// Live Documentation Keyboard Search Engine
function initKeySearch() {
  const searchInput = document.getElementById("key-search");
  if (!searchInput) return;

  searchInput.addEventListener("input", (e) => {
    const query = e.target.value.toLowerCase().trim();
    const tbodies = document.querySelectorAll(".searchable-tbody");

    tbodies.forEach((tbody) => {
      const rows = tbody.querySelectorAll("tr");
      let visibleRowsInTbody = 0;

      rows.forEach((row) => {
        const textContent = row.textContent.toLowerCase();
        if (textContent.includes(query)) {
          row.classList.remove("hidden");
          visibleRowsInTbody++;
        } else {
          row.classList.add("hidden");
        }
      });

      // Hide the entire section if no key bindings match
      const section = tbody.closest(".docs-section");
      if (section) {
        if (visibleRowsInTbody === 0 && query !== "") {
          section.style.display = "none";
        } else {
          section.style.display = "block";
        }
      }
    });
  });
}

// Live Interactive Terminal Mockup Simulator Logic
function switchSimTab(tabType) {
  const tabButtons = document.querySelectorAll(".terminal-tab-btn");
  const simBody = document.getElementById("sim-body");
  if (!simBody) return;

  // Toggle Active Styles on Button clicked
  tabButtons.forEach((btn) => {
    if (btn.innerText.toLowerCase().includes(tabType)) {
      btn.classList.add("active");
    } else {
      btn.classList.remove("active");
    }
  });

  // Populate TUI mockups based on selected state
  if (tabType === "editor") {
    simBody.innerHTML = `
            <div class="tui-title-bar">
                <span>CirPath v1.0 - editor.rs</span>
                <span>Language: EN</span>
            </div>
            <div class="tui-content">
<span class="tui-line-nums">01
02
03
04
05</span>fn main() {
    println!("Welcome to CirPath - Built entirely in Rust!");
    let editor_focus = true;
    if editor_focus {
        println!("Press Ctrl+E to navigate smoothly.");
    }
}</div>
            <div class="tui-status-bar">
                <span>[Read 6 lines]</span>
                <span>Ln 6, Col 1</span>
            </div>
            <div class="tui-shortcuts">
                <div class="tui-shortcut-item"><span>^G</span> Help</div>
                <div class="tui-shortcut-item"><span>^O</span> WriteOut</div>
                <div class="tui-shortcut-item"><span>^R</span> ReadFile</div>
                <div class="tui-shortcut-item"><span>^W</span> Where Is</div>
                <div class="tui-shortcut-item"><span>^\</span> Replace</div>
                <div class="tui-shortcut-item"><span>^_</span> Go To Line</div>
                <div class="tui-shortcut-item"><span>^K</span> Cut Text</div>
                <div class="tui-shortcut-item"><span>^U</span> UnCut</div>
                <div class="tui-shortcut-item"><span>^C</span> Cur Pos</div>
                <div class="tui-shortcut-item"><span>^S</span> Save</div>
                <div class="tui-shortcut-item"><span>^E</span> Next Win</div>
                <div class="tui-shortcut-item"><span>^X</span> Exit</div>
            </div>
        `;
  } else if (tabType === "browser") {
    simBody.innerHTML = `
            <div class="tui-title-bar">
                <span>CirPath v1.0 - File Browser</span>
                <span>Path: /projects/cirpath/</span>
            </div>
            <div class="tui-content" style="line-height: 1.6;">
                <div>  .. (Up to Parent)</div>
                <div class="directory-entry">  📁 src/</div>
                <div class="directory-entry">  📁 tests/</div>
                <div class="file-entry selected-row">  📄 Cargo.toml  (1.2 KB)</div>
                <div class="file-entry">  📄 README.md   (4.8 KB)</div>
                <div class="file-entry">  📄 config.toml (320 B)</div>
            </div>
            <div class="tui-status-bar">
                <span>6 entries (H: Hidden files off)</span>
                <span>Select file & Press Enter</span>
            </div>
            <div class="tui-shortcuts" style="grid-template-columns: repeat(4, 1fr);">
                <div class="tui-shortcut-item"><span>g</span> Go To Path</div>
                <div class="tui-shortcut-item"><span>n</span> New File</div>
                <div class="tui-shortcut-item"><span>N</span> New Folder</div>
                <div class="tui-shortcut-item"><span>r</span> Rename</div>
                <div class="tui-shortcut-item"><span>d</span> Delete</div>
                <div class="tui-shortcut-item"><span>^H</span> Show Hidden</div>
                <div class="tui-shortcut-item"><span>^E</span> Next Win</div>
                <div class="tui-shortcut-item"><span>Esc</span> Cancel</div>
            </div>
        `;
  } else if (tabType === "menu") {
    simBody.innerHTML = `
            <div class="tui-title-bar">
                <span>CirPath v1.0 - Settings Menu</span>
                <span>Config: ~/.config/cirpath/config.toml</span>
            </div>
            <div class="tui-content" style="display: grid; grid-template-columns: 200px 1fr; gap: 10px;">
                <div style="border-right: 1px solid var(--border-color); padding-right: 10px;">
                    <div style="color: var(--accent-yellow); font-weight: bold;">▶ General</div>
                    <div style="color: #666;">  Editor</div>
                    <div style="color: #666;">  File Browser</div>
                    <div style="color: #666;">  Key Bindings</div>
                    <div style="color: #666;">  Config File</div>
                </div>
                <div>
                    <div style="font-weight: bold; margin-bottom: 0.5rem; color: #fff;">General Configuration</div>
                    <div>Language Switch: <span style="background: #222; padding: 2px 5px; color: var(--accent-yellow);">[ EN ]</span> JP</div>
                    <div style="color: #888; font-size: 0.8rem; margin-top: 0.2rem;">Changes will apply instantly across all active views.</div>
                    <div style="margin-top: 10px;">Auto-switch to editor: <span style="color: var(--terminal-green);">Enabled [X]</span></div>
                </div>
            </div>
            <div class="tui-status-bar">
                <span>Enter: Toggle Settings</span>
                <span>Arrow Keys: Navigate</span>
            </div>
            <div class="tui-shortcuts" style="grid-template-columns: repeat(3, 1fr);">
                <div class="tui-shortcut-item"><span>Arrows</span> Navigate</div>
                <div class="tui-shortcut-item"><span>Enter</span> Apply/Toggle</div>
                <div class="tui-shortcut-item"><span>^E</span> Next Win</div>
            </div>
        `;
  }
}
