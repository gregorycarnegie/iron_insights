import type { ArrowResponse, EquipmentOption, LiftType, SexValue } from './types';

// Modern UI control functions
function setToggle(element: HTMLElement, type: 'sex' | 'lift'): boolean {
  // Buttons are type="button"; no default submit to prevent

  // Remove active class from siblings and update aria-checked
  element.parentElement?.querySelectorAll('.toggle-button').forEach(btn => {
    btn.classList.remove('active');
    btn.setAttribute('aria-checked', 'false');
    btn.setAttribute('tabindex', '-1');
  });
  // Add active class to clicked element and update aria-checked
  element.classList.add('active');
  element.setAttribute('aria-checked', 'true');
  element.setAttribute('tabindex', '0');

  // Focus the clicked element
  element.focus();

  // Update global state
  const value = element.getAttribute('data-value');
  if (type === 'sex') {
    window.currentSex = (value as SexValue) || window.currentSex;
    console.log('Updated currentSex to:', window.currentSex);
  } else if (type === 'lift') {
    window.currentLiftType = (value as LiftType) || window.currentLiftType;
    console.log('Updated currentLiftType to:', window.currentLiftType);
  }

  // Update charts when toggle changes if the function exists
  if (typeof window.updateCharts === 'function') {
    window.updateCharts();
  }

  return false;
}

function updateEquipment(): void {
  // Get all checked equipment checkboxes
  const selected: EquipmentOption[] = [];
  if ((document.getElementById('equipment-raw') as HTMLInputElement | null)?.checked) selected.push('Raw');
  if ((document.getElementById('equipment-wraps') as HTMLInputElement | null)?.checked) selected.push('Wraps');
  if ((document.getElementById('equipment-single-ply') as HTMLInputElement | null)?.checked) selected.push('Single-ply');
  if ((document.getElementById('equipment-multi-ply') as HTMLInputElement | null)?.checked) selected.push('Multi-ply');

  window.currentEquipment = selected.length > 0 ? selected : ['Raw'];

  window.updateCharts();
}

function updateAnalytics(): void {
  console.log('updateAnalytics called');
  window.updateCharts();
}

// Handle equipment checkbox logic for modern UI
function setupEquipmentFilters(): void {
  const equipmentCheckboxes = ['equipment-raw', 'equipment-wraps', 'equipment-single-ply', 'equipment-multi-ply'];

  equipmentCheckboxes.forEach(id => {
    const checkbox = document.getElementById(id) as HTMLInputElement | null;
    if (checkbox) {
      checkbox.addEventListener('change', updateEquipment);
    }
  });
}

// Weight class filter setup
function setupWeightClassFilter(): void {
  const weightClassSelect = document.getElementById('weightClass') as HTMLSelectElement | null;
  if (weightClassSelect) {
    weightClassSelect.addEventListener('change', function (e) {
      window.currentWeightClass = (e.target as HTMLSelectElement).value;
      console.log('Weight class changed to:', window.currentWeightClass);
      window.updateCharts();
    });
  }
}

// Time period filter setup
function setupTimePeriodFilter(): void {
  const timePeriodSelect = document.getElementById('timePeriod') as HTMLSelectElement | null;
  if (timePeriodSelect) {
    timePeriodSelect.addEventListener('change', function (e) {
      window.currentTimePeriod = (e.target as HTMLSelectElement).value;
      console.log('Time period changed to:', window.currentTimePeriod);
      window.updateCharts();
    });
  }
}

// Federation filter setup
function setupFederationFilter(): void {
  const federationSelect = document.getElementById('federation') as HTMLSelectElement | null;
  if (federationSelect) {
    federationSelect.addEventListener('change', function (e) {
      window.currentFederation = (e.target as HTMLSelectElement).value;
      console.log('Federation changed to:', window.currentFederation);
      window.updateCharts();
    });
  }
}

// Monitor input changes for debugging
function setupInputDebugger(): void {
  const userLiftInput = document.getElementById('userLift') as HTMLInputElement | null;
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

    userLiftInput.addEventListener('input', function (e) {
      console.log('userLift input changed:', `"${(e.target as HTMLInputElement).value}"`);
    });

    userLiftInput.addEventListener('change', function (e) {
      console.log('userLift change event:', `"${(e.target as HTMLInputElement).value}"`);
    });

    userLiftInput.addEventListener('blur', function (e) {
      console.log('userLift blur event:', `"${(e.target as HTMLInputElement).value}"`);
    });
  }
}

// Mobile sidebar toggle + overlay handling
function toggleSidebar(): void {
  const sidebar = document.getElementById('sidebar');
  const overlay = document.getElementById('sidebarOverlay');
  const toggleButton = document.querySelector('.mobile-menu-toggle') as HTMLElement | null;
  if (!sidebar || !overlay) return;
  const open = !sidebar.classList.contains('mobile-open');
  sidebar.classList.toggle('mobile-open', open);
  overlay.classList.toggle('active', open);
  if (toggleButton) {
    toggleButton.setAttribute('aria-expanded', open.toString());
  }
}

// Keyboard navigation for toggle groups
function setupKeyboardNavigation(): void {
  document.querySelectorAll<HTMLElement>('[role="radiogroup"]').forEach(group => {
    const buttons = group.querySelectorAll<HTMLElement>('[role="radio"]');

    buttons.forEach((button, index) => {
      button.addEventListener('keydown', e => {
        let targetIndex: number | undefined;

        switch (e.key) {
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
function setupEscapeKeyHandling(): void {
  document.addEventListener('keydown', e => {
    if (e.key === 'Escape') {
      const sidebar = document.getElementById('sidebar');
      if (sidebar && sidebar.classList.contains('mobile-open')) {
        toggleSidebar();
        (document.querySelector('.mobile-menu-toggle') as HTMLElement | null)?.focus();
      }
    }
  });
}

// Highlight active nav link
function highlightActiveNav(): void {
  try {
    const path = window.location.pathname.replace(/\/$/, '');
    document.querySelectorAll<HTMLAnchorElement>('.header-nav a').forEach(a => {
      const href = a.getAttribute('href');
      if (!href || href.startsWith('#')) return;
      const cleanHref = href.replace(/\/$/, '');
      if (cleanHref === path || (cleanHref && path.startsWith(cleanHref) && cleanHref !== '')) {
        a.classList.add('active');
      }
    });
  } catch {
    // ignore failures
  }
}

document.addEventListener('DOMContentLoaded', () => {
  // Progressive enhancement setup
  document.body.classList.remove('no-js');
  document.body.classList.add('js');

  // Mark JS as enabled for form submission
  const jsInput = document.getElementById('js-enabled') as HTMLInputElement | null;
  if (jsInput) jsInput.value = '1';

  highlightActiveNav();
  setupEquipmentFilters();
  setupWeightClassFilter();
  setupTimePeriodFilter();
  setupFederationFilter();
  setupKeyboardNavigation();
  setupEscapeKeyHandling();
  setupProgressiveEnhancement();
});

// Progressive enhancement for form elements
function setupProgressiveEnhancement(): void {
  // Hide no-js fallbacks and show enhanced versions
  const noJsElements = document.querySelectorAll<HTMLElement>('.no-js-only');
  const jsElements = document.querySelectorAll<HTMLElement>('.js-only');

  noJsElements.forEach(el => (el.style.display = 'none'));
  jsElements.forEach(el => {
    // Check if it's a toggle-group and set appropriate display
    if (el.classList.contains('toggle-group')) {
      el.style.display = 'flex';
    } else {
      el.style.display = 'block';
    }
  });
}

function toggleDebug(): void {
  window.debugMode = !window.debugMode;
  const debugInfo = document.getElementById('debugInfo') as HTMLElement | null;
  if (!debugInfo) return;
  debugInfo.style.display = window.debugMode ? 'block' : 'none';

  if (window.debugMode && window.lastResponse) {
    showDebugInfo(window.lastResponse);
  }
}

function showDebugInfo(data: ArrowResponse): void {
  if (!window.debugMode) return;

  const debugInfo = document.getElementById('debugInfo') as HTMLElement | null;
  if (!debugInfo) return;
  debugInfo.innerHTML =
    '<strong>Debug Information:</strong><br>' +
    'Raw histogram values: ' +
    data.histogram_data.values.length +
    '<br>' +
    'DOTS histogram values: ' +
    data.dots_histogram_data.values.length +
    '<br>' +
    'Raw scatter points: ' +
    data.scatter_data.x.length +
    '<br>' +
    'DOTS scatter points: ' +
    data.dots_scatter_data.x.length +
    '<br>' +
    'Processing time: ' +
    data.processing_time_ms +
    'ms<br>' +
    'Total records: ' +
    data.total_records +
    '<br>' +
    'User percentile: ' +
    data.user_percentile +
    '<br>' +
    'User DOTS percentile: ' +
    data.user_dots_percentile;
}

// Expose functions to global scope
window.setToggle = setToggle;
window.updateEquipment = updateEquipment;
window.updateAnalytics = updateAnalytics;
window.toggleSidebar = toggleSidebar;
window.toggleDebug = toggleDebug;
window.setupEquipmentFilters = setupEquipmentFilters;
window.setupWeightClassFilter = setupWeightClassFilter;
window.setupTimePeriodFilter = setupTimePeriodFilter;
window.setupFederationFilter = setupFederationFilter;
window.setupInputDebugger = setupInputDebugger;
window.showDebugInfo = showDebugInfo;
