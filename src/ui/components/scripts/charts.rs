use maud::{Markup, PreEscaped};

pub fn render_chart_scripts() -> Markup {
    PreEscaped(r#"
        function showError(chartId, message) {
            const errorElement = document.getElementById(chartId + 'Error');
            if (errorElement) {
                errorElement.style.display = 'block';
                errorElement.textContent = message;
            }
        }
        
        function hideError(chartId) {
            const errorElement = document.getElementById(chartId + 'Error');
            if (errorElement) {
                errorElement.style.display = 'none';
            }
        }
        
        function createPlot(chartId, traces, layout, errorMessage = 'No data available') {
            if (!traces || traces.length === 0 || 
                (traces[0].x && traces[0].x.length === 0) ||
                (traces[0].values && traces[0].values.length === 0)) {
                showError(chartId, errorMessage);
                return false;
            }
            
            hideError(chartId);
            // Enhanced Plotly config for GPU acceleration
            const config = {
                displayModeBar: false,
                staticPlot: false,
                responsive: true,
                webGlRenderer: true
            };
            Plotly.react(chartId, traces, layout, config);
            return true;
        }
        
        // Chart control functions for modern UI
        function changeBins(element, chartId) {
            const bins = parseInt(element.getAttribute('data-bins'));
            currentBinCount = bins;
            
            // Update active state
            element.parentElement.querySelectorAll('.chart-option').forEach(btn => {
                btn.classList.remove('active');
            });
            element.classList.add('active');
            
            // Re-render chart with new bins
            updateCharts();
        }
        
        function switchRankings(element) {
            const type = element.getAttribute('data-type');
            
            // Update active state
            element.parentElement.querySelectorAll('.chart-option').forEach(btn => {
                btn.classList.remove('active');
            });
            element.classList.add('active');
            
            // Update rankings table based on type
            updateRankingsTable(type);
        }
        
        function toggleTrendline(chartId) {
            // Implement trendline toggle
            console.log('Toggle trendline for', chartId);
        }
        
        function togglePoints(chartId) {
            // Implement points toggle
            console.log('Toggle points for', chartId);
        }
        
        function updateRankingsTable(type) {
            // Implementation for updating rankings table
            console.log('Update rankings table with type:', type);
        }
    "#.to_string())
}