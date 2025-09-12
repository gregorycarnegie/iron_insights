use maud::{Markup, PreEscaped};

pub fn render_ui_scripts() -> Markup {
    PreEscaped(r#"
        // Modern UI control functions
        function setToggle(element, type) {
            // Remove active class from siblings and update aria-checked
            element.parentElement.querySelectorAll('.toggle-button').forEach(btn => {
                btn.classList.remove('active');
                btn.setAttribute('aria-checked', 'false');
                btn.setAttribute('tabindex', '-1');
            });
            // Add active class to clicked element and update aria-checked
            element.classList.add('active');
            element.setAttribute('aria-checked', 'true');
            element.setAttribute('tabindex', '0');
            
            // Update global state
            const value = element.getAttribute('data-value');
            if (type === 'sex') {
                currentSex = value;
            } else if (type === 'lift') {
                currentLiftType = value;
            }
            
            // Update charts when toggle changes
            updateCharts();
        }
        
        function updateEquipment() {
            // Get all checked equipment checkboxes
            currentEquipment = [];
            if (document.getElementById('equipment-raw').checked) currentEquipment.push("Raw");
            if (document.getElementById('equipment-wraps').checked) currentEquipment.push("Wraps");
            if (document.getElementById('equipment-single-ply').checked) currentEquipment.push("Single-ply");
            if (document.getElementById('equipment-multi-ply').checked) currentEquipment.push("Multi-ply");
            
            // Default to Raw if nothing selected
            if (currentEquipment.length === 0) {
                currentEquipment.push("Raw");
            }
            
            updateCharts();
        }
        
        function updateAnalytics() {
            console.log('updateAnalytics called');
            updateCharts();
        }
        
        // Handle equipment checkbox logic for modern UI
        function setupEquipmentFilters() {
            const equipmentCheckboxes = [
                'equipment-raw', 'equipment-wraps', 'equipment-single-ply', 'equipment-multi-ply'
            ];
            
            equipmentCheckboxes.forEach(id => {
                const checkbox = document.getElementById(id);
                if (checkbox) {
                    checkbox.addEventListener('change', updateEquipment);
                }
            });
        }
        
        // Monitor input changes for debugging
        function setupInputDebugger() {
            const userLiftInput = document.getElementById('userLift');
            if (userLiftInput) {
                console.log('Setting up input debugger for userLift');
                console.log('Input type:', userLiftInput.type);
                console.log('Current value:', `"${userLiftInput.value}"`);
                
                // Change input type to text to allow expressions like "340+270+190"
                if (userLiftInput.type === 'number') {
                    console.log('Changing input type from number to text to allow expressions');
                    userLiftInput.type = 'text';
                    userLiftInput.placeholder = 'Enter lift or sum (e.g. 340+270+190)';
                }
                
                userLiftInput.addEventListener('input', function(e) {
                    console.log('userLift input changed:', `"${e.target.value}"`);
                });
                
                userLiftInput.addEventListener('change', function(e) {
                    console.log('userLift change event:', `"${e.target.value}"`);
                });
                
                userLiftInput.addEventListener('blur', function(e) {
                    console.log('userLift blur event:', `"${e.target.value}"`);
                });
            }
        }

        // Mobile sidebar toggle + overlay handling
        function toggleSidebar() {
            const sidebar = document.getElementById('sidebar');
            const overlay = document.getElementById('sidebarOverlay');
            const toggleButton = document.querySelector('.mobile-menu-toggle');
            if (!sidebar || !overlay) return;
            const open = !sidebar.classList.contains('mobile-open');
            sidebar.classList.toggle('mobile-open', open);
            overlay.classList.toggle('active', open);
            if (toggleButton) {
                toggleButton.setAttribute('aria-expanded', open.toString());
            }
        }

        // Keyboard navigation for toggle groups
        function setupKeyboardNavigation() {
            document.querySelectorAll('[role="radiogroup"]').forEach(group => {
                const buttons = group.querySelectorAll('[role="radio"]');
                
                buttons.forEach((button, index) => {
                    button.addEventListener('keydown', (e) => {
                        let targetIndex;
                        
                        switch(e.key) {
                            case 'ArrowLeft':
                            case 'ArrowUp':
                                e.preventDefault();
                                targetIndex = index > 0 ? index - 1 : buttons.length - 1;
                                break;
                            case 'ArrowRight':
                            case 'ArrowDown':
                                e.preventDefault();
                                targetIndex = index < buttons.length - 1 ? index + 1 : 0;
                                break;
                            case 'Home':
                                e.preventDefault();
                                targetIndex = 0;
                                break;
                            case 'End':
                                e.preventDefault();
                                targetIndex = buttons.length - 1;
                                break;
                            case ' ':
                            case 'Enter':
                                e.preventDefault();
                                button.click();
                                return;
                            default:
                                return;
                        }
                        
                        if (targetIndex !== undefined) {
                            buttons[targetIndex].focus();
                            buttons[targetIndex].click();
                        }
                    });
                });
            });
        }

        // Escape key handling
        function setupEscapeKeyHandling() {
            document.addEventListener('keydown', (e) => {
                if (e.key === 'Escape') {
                    const sidebar = document.getElementById('sidebar');
                    if (sidebar && sidebar.classList.contains('mobile-open')) {
                        toggleSidebar();
                        document.querySelector('.mobile-menu-toggle')?.focus();
                    }
                }
            });
        }

        // Highlight active nav link
        function highlightActiveNav() {
            try {
                const path = window.location.pathname.replace(/\/$/, '');
                document.querySelectorAll('.header-nav a').forEach(a => {
                    const href = a.getAttribute('href');
                    if (!href || href.startsWith('#')) return;
                    const cleanHref = href.replace(/\/$/, '');
                    if (cleanHref === path || (cleanHref && path.startsWith(cleanHref) && cleanHref !== '')) {
                        a.classList.add('active');
                    }
                });
            } catch (_) {}
        }

        document.addEventListener('DOMContentLoaded', () => {
            highlightActiveNav();
            setupEquipmentFilters();
            setupKeyboardNavigation();
            setupEscapeKeyHandling();
        });

        function toggleDebug() {
            debugMode = !debugMode;
            const debugInfo = document.getElementById('debugInfo');
            debugInfo.style.display = debugMode ? 'block' : 'none';
            
            if (debugMode && lastResponse) {
                showDebugInfo(lastResponse);
            }
        }
        
        function showDebugInfo(data) {
            if (!debugMode) return;
            
            const debugInfo = document.getElementById('debugInfo');
            debugInfo.innerHTML = '<strong>Debug Information:</strong><br>' +
                'Raw histogram values: ' + data.histogram_data.values.length + '<br>' +
                'DOTS histogram values: ' + data.dots_histogram_data.values.length + '<br>' +
                'Raw scatter points: ' + data.scatter_data.x.length + '<br>' +
                'DOTS scatter points: ' + data.dots_scatter_data.x.length + '<br>' +
                'Processing time: ' + data.processing_time_ms + 'ms<br>' +
                'Total records: ' + data.total_records + '<br>' +
                'User percentile: ' + data.user_percentile + '<br>' +
                'User DOTS percentile: ' + data.user_dots_percentile;
        }
    "#.to_string())
}
