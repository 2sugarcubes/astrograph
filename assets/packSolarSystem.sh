echo "{
  \"rootBody\": $(<"solar-system.json"),
  \"observatories\": $(<"solar-system.observatories.json"),
  \"outputFileRoot\": \"./output/\"
}" | jq >"solar-system.program.json"
