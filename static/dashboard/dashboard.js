document.addEventListener("DOMContentLoaded", function () {
    fetch("/dashboard/donuts", {
        method: "GET",
        credentials: "include",
    })
    .then(response => {
        if (!response.ok) {
            console.log("HTTP error " + response.status);
        }
        return response.json();
    })
    .then(data => {
        var subspecGraphDiv = document.getElementById("container-b1");
        var modalityGraphDiv = document.getElementById("container-c1");

        const COLORS = [
            "#003f5c",
            "#444e86",
            "#955196",
            "#dd5182",
            "#ff6e54",
            "#ffa600",
        ]
        var allArray = {
            subspecialty: data.subspecialty.categories.map((category, index) => {
                return {
                    name: category,
                    y: data.subspecialty.data[index],
                    color: COLORS[index]
                }
            }),
            modality: data.modality.categories.map((category, index) => {
                return {
                    name: category,
                    y: data.modality.data[index],
                    color: COLORS[index]
                }
            }),
        }

        var assocColors = {
            ...data.subspecialty.categories.reduce((acc, category, index) => {
                acc[category] = COLORS[index]
                return acc
            }, {}),
            ...data.modality.categories.reduce((acc, category, index) => {
                acc[category] = COLORS[index]
                return acc
            }, {})
        };
        
        // Update the colors of the donut chart legend bullets
        document.querySelectorAll("span.plot-categories-list-category").forEach((element) => {
            const category = element.textContent.trim();
            console.log(category);
            element.style.setProperty("--bullet-color", assocColors[category]);
        });

        const highchartOptions = {
            chart: { 
                type: "pie",
                reflow: false,
                backgroundColor: "rgba(255, 255, 255, 0)",
            },
            plotOptions: {
                pie: {
                    shadow: false,
                    center: ["50%", "50%"],
                    enableMouseTracking: false,
                    dataLabels: {
                        enabled: true,
                        format: "{point.percentage: .1f} %",
                        distance: 5,
                        style: {
                            fontWeight: "bold",
                            color: "gray",
                            textOutline: "none",
                            fontSize: "1rem"
                        }
                    },
                }
            },
            tooltip: { enabled: false },
            credits: { enabled: false },
            title: { text: "" },
        }
        Highcharts.chart(subspecGraphDiv, {
            ...highchartOptions,
            series: [{
                name: "Subspecialty",
                data: allArray.subspecialty,
                innerSize: "60%"
            }]
        })
        Highcharts.chart(modalityGraphDiv, {
            ...highchartOptions,
            series: [{
                name: "Modality",
                data: allArray.modality,
                innerSize: "60%"
            }],
        })
    })
})
