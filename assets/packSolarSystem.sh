echo "{
  \"rootBody\": $(<"solar-system.json"),
  \"observatories\": $(<"solar-system.observatories.json")
}" | jq >"solar-system.program.json"
