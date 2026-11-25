// Test-only known-correct C++ fixture for min-add.
long long fewest_inserts(std::string inscription) {
    long long openLayers = 0;
    long long stranded = 0;
    for (char rune : inscription) {
        if (rune == '(') {
            openLayers += 1;
        } else if (openLayers > 0) {
            openLayers -= 1;
        } else {
            stranded += 1;
        }
    }
    return openLayers + stranded;
}

JSONValue counterspell_call(const std::vector<JSONValue>& args) {
    return to_json(fewest_inserts(from_json<std::string>(args[0])));
}
