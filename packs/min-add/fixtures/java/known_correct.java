// Test-only known-correct Java fixture for min-add.
public class Solution {
    public static long fewest_inserts(String inscription) {
        long openLayers = 0;
        long stranded = 0;
        for (int index = 0; index < inscription.length(); index += 1) {
            if (inscription.charAt(index) == '(') {
                openLayers += 1;
            } else if (openLayers > 0) {
                openLayers -= 1;
            } else {
                stranded += 1;
            }
        }
        return openLayers + stranded;
    }
}
