// Test-only known-correct Java fixture for undo-ritual.
public class Solution {
    public static String surviving_text(String chant) {
        StringBuilder written = new StringBuilder();
        for (int index = 0; index < chant.length(); index += 1) {
            char rune = chant.charAt(index);
            if (rune == '#') {
                if (written.length() > 0) {
                    written.setLength(written.length() - 1);
                }
            } else {
                written.append(rune);
            }
        }
        return written.toString();
    }
}
