import init, {
  //initThreadPool,
  generate_observations_from_json,
  generate_universe,
} from "./pkg/astrograph.js";

async function loadJsonFile(url, callback) {
  fetch(url)
    .then((response) => response.json())
    .then((json) => callback(json));
}

const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

var frame = document.getElementById("slides");
const firstNavButton = document.getElementById("navButtonPrev");

window.draw_observation = function draw_observation(time, svgData) {
  const lines = svgData.split("\n").length;

  var dataURL =
    "data:image/svg+xml;charset=utf-8," + encodeURIComponent(svgData);

  if (lines > 40) {
    console.debug(time, lines);
  }
  var slide = document.createElement("img");
  slide.src = dataURL;
  slide.classList.add("slide");
  frame.insertBefore(slide, firstNavButton);
};

window.simulate = async function simulate() {
  /*

  console.info("Creating a whole universe");
  const root = generate_universe();
  const observatories = [
    {
      body_id: [0, 1],
      location: { r: 1.0, theta: 2.146716234, phi: 2.587676113 },
    },
  ];
  
  console.info("Generating 10 observatons");
  generate_observations_from_json(
    JSON.stringify(root),
    JSON.stringify(observatories),
    BigInt(0),
    BigInt(121),
    12,
  );

  await sleep(10500); */
  await loadJsonFile("/assets/solar-system.json", async function (solarsystem) {
    console.info("Loaded Solar System");
    const observatories = [
      {
        body_id: [2],
        location: { r: 1.0, theta: 2.146716234, phi: 2.587676113 },
      },
    ];
    /*await loadJsonFile(
      "/assets/solar-system.observatories.json",
      function (observatories) {
        console.info("Loaded observatories");
        console.info("Generating 100 observations"); */
    frame.querySelectorAll(":scope > img").forEach((e) => e.remove());
    generate_observations_from_json(
      JSON.stringify(solarsystem),
      JSON.stringify(observatories),
      BigInt(0),
      BigInt(100),
      1,
    );

    /*
      //updateLoop(BigInt(0), 1, BigInt(100));
      },
    );*/
  });
};

console.debug(frame);
console.debug(firstNavButton);

console.debug("Initializing wasm modules");
await init();

// TODO: add support for web workers

//await initThreadPool(navigator.hardwareConcurrency);
console.debug("Finished initializing wasm modules");
