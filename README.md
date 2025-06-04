# Binary Orbit 3D Visualizer 🌌

**Check it out live: [https://ryan-tock.github.io/binary-orbits/](https://ryan-tock.github.io/binary-orbits/)**

Welcome to the Binary Orbit 3D Visualizer! This graph is for plotting existing (or new) binary star data in a way that can be easily visualized.

---

## 🌟 What Can It Do? (Key Features)

* **See Orbits in 3D**: Get a sense of a binary system in three-dimensional space.
* **Fit Your Data**: Crunch observational data and find out the best-fit orbit, displaying the key parameters.
* **Interactivity**:
    * **Sample Orbits**: The data for 4 actual orbital systems are included with this site.
    * **Upload Data**: This tool supports uploading your own binary star data.
    * **Change Faulty Data**:
        * **Remove Point**: You can remove outliers from the visualizations and calculations
        * **Highlight Point**: Make a specific data point stand out by coloring it black.
        * **Flip Coordinates**: Historical data often contains points with some coordinates flipped, so flip them back!
* **Get the Results**: Once optimized, you'll see the orbital parameters that describe the orbit.

---

* **Set Period Bounds**: You must give the tool a range containing the true period for it to function.
## 🚀 Using The Tool

### 1. Getting Data on the Screen

* **Using The Sample Data**:
    * Ensure the "Use sample data" checkbox is tucked.
    * Pick an example orbit from the dropdown list.

* **Using Your Own Data**:
    * To see the correct data format, load some sample data and then click the "Download updated data file" button. That'll give you a template.
    * Once your file is ready, use the file browser (it will appear when you uncheck "Use sample data") to load your data.

**Important Note**: At this stage, you'll only see the raw data points plotted in the 3D space. The orbit hasn't been fitted yet!

### 2. Fitting Data
* **Ensure Period Bounds are Set Correctly**
    * The default period bounds are 2-40 years. Many binary systems are well outside of this range, so ensure you have the bounds set to an appropriate timeframe.
    * The units are always years, so make sure units are converted before putting them in.
* **Hit "Optimzie Orbit!"**

### 3. Tweaking Data

* **Choose Appropriate Setting**
    * Do Nothing
    * Remove Point       Removes points.
    * Highlight point    Makes points black to distinguish them.
    * Flip X             Negates the X coordinate of the point.
    * Flip Y             Negates the Y coordinate of the point.
    * Flip X and Y       Negates the X and Y coordinates of the point.
* **Click any point on the graph.**
    * When all data is adjusted. It is highly recommended that you re-fit the data using the "Optimize Orbit!" button.

### 4. Download Updated Data and Read Parameters

* **Download Updated Data**
    * Click the "Download updated data file" button.
* **Read Parameters**
    * Read all 7 orbital parametrs at the bottom section of the page.