// Test-only known-correct Java fixture for split-ward.
public class Solution {
    public static long can_segment(String words, String s) {
        java.util.Set<String> ws = new java.util.HashSet<>();
        for (String w : words.split(",", -1)) {
            if (!w.isEmpty()) {
                ws.add(w);
            }
        }
        boolean[] ok = new boolean[s.length() + 1];
        ok[0] = true;
        for (int i = 0; i <= s.length(); i += 1) {
            if (!ok[i]) {
                continue;
            }
            for (String w : ws) {
                if (s.startsWith(w, i)) {
                    ok[i + w.length()] = true;
                }
            }
        }
        return ok[s.length()] ? 1 : 0;
    }
}
