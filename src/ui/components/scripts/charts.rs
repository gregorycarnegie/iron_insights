use maud::{Markup, PreEscaped};

pub fn render_chart_scripts() -> Markup {
    PreEscaped(r#"
        // Cache for chart configurations to avoid recreating them
        const CHART_CONFIGS = new Map();

        function getChartConfig(chartId, layout) {
            const cacheKey = `${chartId}-${JSON.stringify(layout)}`;

            if (!CHART_CONFIGS.has(cacheKey)) {
                CHART_CONFIGS.set(cacheKey, {
                    displayModeBar: false,
                    staticPlot: false,
                    responsive: true,
                    webGlRenderer: true
                });
            }

            return CHART_CONFIGS.get(cacheKey);
        }

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
        
        // Use requestAnimationFrame for chart updates to optimize rendering
        let chartUpdateFrames = new Map();

        function createPlot(chartId, traces, layout, errorMessage = 'No data available') {
            if (!traces || traces.length === 0 ||
                (traces[0].x && traces[0].x.length === 0) ||
                (traces[0].values && traces[0].values.length === 0)) {
                showError(chartId, errorMessage);
                hideChartSkeleton(chartId);
                return false;
            }

            hideError(chartId);

            // Cancel any pending chart update for this chart
            if (chartUpdateFrames.has(chartId)) {
                cancelAnimationFrame(chartUpdateFrames.get(chartId));
            }

            // Use requestAnimationFrame for smooth rendering
            const frameId = requestAnimationFrame(() => {
                // Use cached config for better performance
                const config = getChartConfig(chartId, layout);

                try {
                    // Use Plotly.react for efficient updates (reuses existing DOM)
                    Plotly.react(chartId, traces, layout, config);
                    // Hide skeleton after successful chart render
                    hideChartSkeleton(chartId);
                    chartUpdateFrames.delete(chartId);
                } catch (error) {
                    console.error('Error creating plot:', error);
                    showError(chartId, 'Failed to render chart');
                    hideChartSkeleton(chartId);
                    chartUpdateFrames.delete(chartId);
                }
            });

            chartUpdateFrames.set(chartId, frameId);
            return true;
        }

        function hideChartSkeleton(chartId) {
            // Find skeleton containers related to this chart
            const skeletons = document.querySelectorAll(`.skeleton-container[data-chart="${chartId}"], #${chartId}-skeleton`);
            const contents = document.querySelectorAll(`.content-container[data-chart="${chartId}"], #${chartId}-content`);
            
            skeletons.forEach(skeleton => {
                skeleton.classList.add('loaded');
            });
            
            contents.forEach(content => {
                content.classList.add('loaded');
            });

            // Also hide any generic chart skeletons near the chart element
            const chartElement = document.getElementById(chartId);
            if (chartElement) {
                const nearbySkeletons = chartElement.parentElement?.querySelectorAll('.skeleton-chart, .skeleton-container');
                nearbySkeletons?.forEach(skeleton => {
                    skeleton.classList.add('loaded');
                    skeleton.style.opacity = '0';
                    skeleton.style.pointerEvents = 'none';
                });
            }
        }
        
        // Global state for chart interactions
        let selectedDataPoints = new Set();
        let brushSelection = null;
        let chartFilters = new Map();

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

        // Crossfilter-style chart linking functionality
        let crossfilteringSetupAttempts = 0;
        const MAX_CROSSFILTERING_ATTEMPTS = 10; // Maximum 10 seconds of retries

        function setupChartCrossfiltering() {
            const chartIds = ['weightDistribution', 'dotsDistribution', 'bodyweightScatter', 'dotsScatter'];
            let allChartsReady = true;
            let readyCount = 0;

            chartIds.forEach(chartId => {
                const chartElement = document.getElementById(chartId);
                if (!chartElement || !chartElement._fullLayout) {
                    allChartsReady = false;
                    return;
                }
                readyCount++;

                // Add selection event handlers using Plotly's event system
                try {
                    // Remove existing listeners to avoid duplicates
                    chartElement.removeAllListeners && chartElement.removeAllListeners('plotly_selected');
                    chartElement.removeAllListeners && chartElement.removeAllListeners('plotly_deselect');
                    chartElement.removeAllListeners && chartElement.removeAllListeners('plotly_relayout');

                    chartElement.on('plotly_selected', function(eventData) {
                        handleChartSelection(chartId, eventData);
                    });

                    chartElement.on('plotly_deselect', function() {
                        clearChartSelection(chartId);
                    });

                    // Add brush selection for histograms
                    if (chartId.includes('Distribution')) {
                        chartElement.on('plotly_relayout', function(eventData) {
                            handleHistogramBrush(chartId, eventData);
                        });
                    }

                    console.log(`Crossfiltering setup complete for ${chartId}`);
                } catch (error) {
                    console.error(`Failed to setup crossfiltering for ${chartId}:`, error);
                }
            });

            // Retry logic with maximum attempts
            if (!allChartsReady) {
                crossfilteringSetupAttempts++;

                if (crossfilteringSetupAttempts < MAX_CROSSFILTERING_ATTEMPTS) {
                    console.log(`Charts not ready for crossfiltering (${readyCount}/${chartIds.length} ready), retrying... (attempt ${crossfilteringSetupAttempts}/${MAX_CROSSFILTERING_ATTEMPTS})`);
                    setTimeout(() => setupChartCrossfiltering(), 1000);
                } else {
                    console.warn(`⚠️ Crossfiltering setup timed out after ${MAX_CROSSFILTERING_ATTEMPTS} attempts. ${readyCount}/${chartIds.length} charts ready.`);
                }
            } else {
                console.log(`✅ All ${chartIds.length} charts ready for crossfiltering`);
                crossfilteringSetupAttempts = 0; // Reset for future calls
            }
        }

        function handleChartSelection(sourceChartId, eventData) {
            if (!eventData || !eventData.points) return;

            // Extract selected data points
            const selectedPoints = eventData.points.map(point => ({
                x: point.x,
                y: point.y,
                pointIndex: point.pointIndex,
                traceIndex: point.traceIndex
            }));

            // Store selection for this chart
            chartFilters.set(sourceChartId, selectedPoints);

            // Update other charts to highlight filtered data
            updateLinkedCharts(sourceChartId, selectedPoints);
        }

        function clearChartSelection(sourceChartId) {
            chartFilters.delete(sourceChartId);

            // Reset all charts to show full data
            const chartIds = ['weightDistribution', 'dotsDistribution', 'bodyweightScatter', 'dotsScatter'];
            chartIds.forEach(chartId => {
                if (chartId !== sourceChartId) {
                    resetChartHighlight(chartId);
                }
            });
        }

        function handleHistogramBrush(chartId, eventData) {
            // Handle brushing on histograms for range selection
            if (eventData['xaxis.range[0]'] !== undefined && eventData['xaxis.range[1]'] !== undefined) {
                const range = {
                    min: eventData['xaxis.range[0]'],
                    max: eventData['xaxis.range[1]']
                };

                // Filter other charts based on this range
                applyRangeFilter(chartId, range);
            }
        }

        function updateLinkedCharts(sourceChartId, selectedPoints) {
            const chartIds = ['weightDistribution', 'dotsDistribution', 'bodyweightScatter', 'dotsScatter'];

            chartIds.forEach(chartId => {
                if (chartId !== sourceChartId) {
                    highlightDataInChart(chartId, selectedPoints);
                }
            });
        }

        function highlightDataInChart(chartId, selectedPoints) {
            const chartElement = document.getElementById(chartId);
            if (!chartElement || !chartElement.data) return;

            // Create highlight overlay
            const updates = {
                'marker.opacity': chartElement.data.map(trace => {
                    if (!trace.x || !trace.y) return trace.marker?.opacity || 1;

                    return trace.x.map((x, i) => {
                        const y = trace.y[i];
                        // Check if this point matches any selected points
                        const isSelected = selectedPoints.some(sp =>
                            Math.abs(sp.x - x) < 0.01 && Math.abs(sp.y - y) < 0.01
                        );
                        return isSelected ? 1 : 0.3;
                    });
                })
            };

            Plotly.restyle(chartId, updates);
        }

        function resetChartHighlight(chartId) {
            const chartElement = document.getElementById(chartId);
            if (!chartElement || !chartElement.data) return;

            const updates = {
                'marker.opacity': chartElement.data.map(trace => {
                    if (trace.type === 'histogram') return 1;
                    return Array.isArray(trace.x) ? new Array(trace.x.length).fill(0.6) : 1;
                })
            };

            Plotly.restyle(chartId, updates);
        }

        function applyRangeFilter(sourceChartId, range) {
            // Apply range filter to scatter plots based on histogram selection
            const scatterCharts = ['bodyweightScatter', 'dotsScatter'];

            scatterCharts.forEach(chartId => {
                if (chartId !== sourceChartId) {
                    filterScatterByRange(chartId, range, sourceChartId);
                }
            });
        }

        function filterScatterByRange(chartId, range, sourceChartId) {
            const chartElement = document.getElementById(chartId);
            if (!chartElement || !chartElement.data) return;

            // Determine which axis to filter based on source chart
            let filterAxis = 'y'; // default to y-axis (weight/DOTS values)
            if (sourceChartId === 'bodyweightScatter' || sourceChartId === 'dotsScatter') {
                filterAxis = 'x'; // filter by bodyweight
            }

            const updates = {
                'marker.opacity': chartElement.data.map(trace => {
                    if (!trace.x || !trace.y) return trace.marker?.opacity || 1;

                    return trace.x.map((x, i) => {
                        const value = filterAxis === 'x' ? x : trace.y[i];
                        const inRange = value >= range.min && value <= range.max;
                        return inRange ? 0.8 : 0.2;
                    });
                })
            };

            Plotly.restyle(chartId, updates);
        }

        // High-quality export functionality
        async function exportChart(chartId, format = 'png', filename = null) {
            const chartElement = document.getElementById(chartId);
            if (!chartElement) {
                console.error(`Chart ${chartId} not found`);
                return;
            }

            const exportFilename = filename || `${chartId}_export_${new Date().toISOString().slice(0, 10)}`;

            try {
                let config = {
                    format: format,
                    width: 1200,
                    height: 800,
                    scale: 2 // High DPI for crisp exports
                };

                // Use Plotly's built-in export with high quality settings
                const imageData = await Plotly.toImage(chartElement, config);

                // Create download link
                const link = document.createElement('a');
                link.download = `${exportFilename}.${format}`;
                link.href = imageData;
                document.body.appendChild(link);
                link.click();
                document.body.removeChild(link);

                console.log(`Exported ${chartId} as ${format}`);
            } catch (error) {
                console.error('Export failed:', error);
                alert(`Failed to export chart: ${error.message}`);
            }
        }

        // Export all charts as individual files
        async function exportAllCharts(format = 'png') {
            const chartIds = ['weightDistribution', 'dotsDistribution', 'bodyweightScatter', 'dotsScatter'];
            const timestamp = new Date().toISOString().slice(0, 10);

            console.log(`Starting bulk export of all charts as ${format}...`);

            for (let i = 0; i < chartIds.length; i++) {
                const chartId = chartIds[i];
                const chartElement = document.getElementById(chartId);
                if (chartElement && chartElement._fullLayout) {
                    try {
                        console.log(`Exporting chart ${i + 1}/4: ${chartId}`);
                        await exportChart(chartId, format, `iron_insights_${chartId}_${timestamp}`);
                        // Small delay between exports to prevent browser issues
                        await new Promise(resolve => setTimeout(resolve, 800));
                    } catch (error) {
                        console.error(`Failed to export ${chartId}:`, error);
                        alert(`Failed to export ${chartId}: ${error.message}`);
                    }
                } else {
                    console.warn(`Chart ${chartId} not ready for export`);
                }
            }

            console.log(`Bulk export complete! Downloaded ${chartIds.length} ${format} files.`);
        }

        // Export current data as CSV
        function exportDataAsCSV() {
            if (!lastResponse || !lastResponse.scatter_data) {
                alert('No data available to export');
                return;
            }

            const data = lastResponse.scatter_data;
            const headers = ['Bodyweight', 'Weight', 'Sex', 'DOTS_Score'];
            const rows = [];

            // Add header row
            rows.push(headers.join(','));

            // Add data rows
            for (let i = 0; i < data.x.length; i++) {
                const dotsData = lastResponse.dots_scatter_data;
                const dotsScore = dotsData && dotsData.y[i] ? dotsData.y[i] : '';

                rows.push([
                    data.x[i] || '',
                    data.y[i] || '',
                    data.sex[i] || '',
                    dotsScore
                ].join(','));
            }

            const csvContent = rows.join('\n');
            const blob = new Blob([csvContent], { type: 'text/csv' });
            const url = window.URL.createObjectURL(blob);

            const link = document.createElement('a');
            link.href = url;
            link.download = `iron_insights_data_${new Date().toISOString().slice(0, 10)}.csv`;
            document.body.appendChild(link);
            link.click();
            document.body.removeChild(link);

            window.URL.revokeObjectURL(url);
        }

        // Enhanced chart creation with export and interaction setup
        function createPlotEnhanced(chartId, traces, layout, errorMessage = 'No data available') {
            const success = createPlot(chartId, traces, layout, errorMessage);

            if (success) {
                // Setup interactions after chart is created
                setTimeout(() => {
                    setupChartInteractions(chartId);
                }, 100);
            }

            return success;
        }

        function setupChartInteractions(chartId) {
            const chartElement = document.getElementById(chartId);
            if (!chartElement || !chartElement._fullLayout) return;

            // Enable selection and brush tools
            const isHistogram = chartId.includes('Distribution');

            try {
                if (isHistogram) {
                    // Enable brush selection for histograms
                    Plotly.relayout(chartId, {
                        dragmode: 'select',
                        selectdirection: 'horizontal'
                    });
                } else {
                    // Enable lasso/box select for scatter plots
                    Plotly.relayout(chartId, {
                        dragmode: 'select'
                    });
                }

                // Add double-click to reset selection using Plotly's event system
                chartElement.on('plotly_doubleclick', function() {
                    clearChartSelection(chartId);
                });

                console.log(`Chart interactions setup for ${chartId}`);
            } catch (error) {
                console.error(`Failed to setup interactions for ${chartId}:`, error);
            }
        }

        // Export dropdown functionality
        function toggleExportDropdown(button) {
            const dropdown = button.parentElement;
            const menu = dropdown.querySelector('.export-menu');
            const isVisible = menu.style.display === 'block';

            // Close all other export dropdowns
            document.querySelectorAll('.export-menu').forEach(m => {
                m.style.display = 'none';
            });

            // Toggle this dropdown
            menu.style.display = isVisible ? 'none' : 'block';

            // Close dropdown when clicking outside
            if (!isVisible) {
                setTimeout(() => {
                    document.addEventListener('click', function closeDropdown(e) {
                        if (!dropdown.contains(e.target)) {
                            menu.style.display = 'none';
                            document.removeEventListener('click', closeDropdown);
                        }
                    });
                }, 0);
            }
        }
    "#.to_string())
}
