# Astrograph Wasm Target

[![License](https://img.shields.io/github/license/2sugarcubes/astrograph)](https://github.com/2sugarcubes/astrograph/LICENSE.txt)

[![codecov](https://codecov.io/gh/2sugarcubes/astrograph/branch/dev/graph/badge.svg?token=E27GPTMWQY)](https://codecov.io/github/2sugarcubes/astrograph)
[![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/2sugarcubes/astrograph/tests)](https://github.com/2sugarcubes/astrograph/actions)
[![Total commits](https://img.shields.io/github/commit-activity/t/2sugarcubes/astrograph)]

[![Open Issues](https://img.shields.io/github/issues/2sugarcubes/astrograph)](https://github.com/2sugarcubes/astrograph/issues)

## Building

You can run the following command to compile the library to web assembly (WASM),
this command will build typescript, and JavaScript bindings.

```sh
wasm-pack build --target web --release
```

This function will output a folder with files that will setup the WASM library.
To use this library you can look at [web/index.js](../web/index.js), or use
this minimal code.

```JavaScript
import init, {
  generate_observations_from_json,
  generate_universe,
} from "/pkg/astrograph.js";

/* TODO: select the node that images will live inside */
const frame_parent = document.getElementById("SOME ID");

// Converts from a string representation of a SVG file to a SVG image
// It is critical this function is exported as `window.draw_observation`
// and has the signature (num, string)
window.draw_observation = function draw_observation(time, svgData) {
  // Encode the SVG file to a way that an img could understand
  var dataURL =
    "data:image/svg+xml;charset=utf-8," + encodeURIComponent(svgData);

  // Create an image that will display this slide
  var slide = document.createElement("img");
  frame.src = dataURL;
  frame.classList.add("Optional_Class");
  frame_parent.appendChild(frame);
};

window.simulate = async function simulate() {
  console.info("Creating a whole universe");
  // Generates a universe with 1,000,000 stars
  const root = generate_universe();
  // An Example observation that would be where the Parkes Observatory is on earth
  const observatories = [
    {
      body_id: [0, 1],
      location: { r: 1.0, theta: 2.146716234, phi: 2.587676113 },
    },
  ];

  console.info("Generating 10 observatons");
  // Creates observations at T=[0, 12, 24, 36, 48, ..., 120]
  generate_observations_from_json(
    // JSON representation of the root
    JSON.stringify(root),
    // JSON representation of the observatory/observatories
    JSON.stringify(observatories),
    // Start time (needs to be a BigInt)
    BigInt(0),
    // End time (not included in observations)
    BigInt(132),
    // Step size (time between observations)
    12,
  );
}

// Load the wasm modules (it can take some time)
await init();
```

Then in your HTML add this to the head.

```html
<script type="module" src="PATH/TO/WSAM-LOADER.js" async></script>
```

And then somewhere in the body you can have a button like this to call the simulator.

```html
<button type="button" onclick="simulate()">Start Simulation</button>
```

### Example

See [web/index.html](../web/) for a bare-bones page that implements the
functions in this workspace in a kind of buggy slideshow.
