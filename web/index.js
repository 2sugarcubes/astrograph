import init, {
  generate_observations_from_json,
  generate_universe,
} from "./pkg/astrolabe.js";

async function loadJsonFile(url, callback) {
  fetch(url)
    .then((response) => response.json())
    .then((json) => callback(json));
}

window.draw_observation = function draw_observation(time, svg_data) {
  console.debug(time, svg_data.split("\n").length);
};

window.simulate = async function simulate() {
  await init();

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
    BigInt(10),
    100,
  );

  await loadJsonFile(
    "./assets/solar-system.json",
    async function (solarsystem) {
      console.info("Loaded Solar System");
      await loadJsonFile(
        "./assets/solar-system.observatories.json",
        function (observatories) {
          console.info("Loaded observatories");
          console.info("Generating 100 observations");
          generate_observations_from_json(
            JSON.stringify(solarsystem),
            JSON.stringify(observatories),
            BigInt(0),
            BigInt(100),
            1,
          );
        },
      );
    },
  );
};
