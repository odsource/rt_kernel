# rt_kernel
Realtime kernel based on the blog posts of Philipp Oppermann and extended by an EDF Scheduler approach.

Der Branch `master` enthält die aktuelle Version des Projekts!

## memory.rs
Für die Speicherverwaltung (Stack) wurde ein `struct StackFrame` erstellt, welches den Stackbereich jedes Threads darstellt. Der Bereich wird über zwei virtuelle Adressen (Start und Ende) abgebildet. Um solch einen Stackbereich zu erhalten muss die Funktion `get_stack_frame(mapper, frame_allocator)` aufgerufen werden. Dies passiert bei jeder Threaderstellung. 

Die Implementierung von `get_stack_frame()` ist ähnlich zur Heaperstellung des Blogs. Es wird ein allgemeiner Stack Start festgelegt (Adresse: 0x_8888_8888_0000). Bei jeder Threaderstellung wird nun ausgehend von der vorherigen Stackendadresse ein neuer Stackbereich erstellt. Um Probleme durch überlappende Adressen zu vermeiden wird immer ein Offset von einer Page hinzugefügt. Dies führt zwar zu Fragmentierung sichert aber dahingehend auch die Speicherbereiche einigermaßen ab. 

Um den Frame in den Speicher zu laden und ihn auch zugreifbar zu machen werden die Flags `PageTableFlags::PRESENT` und `PageTableFlags::WRITABLE` gesetzt. Bzw. wird der Bereich der virtuellen Adressen `stack_start` bis `stack_end` in Pages unterteilt und für jede Page ein Mapping auf einen zugehörigen Frame gemacht. Hierbei werden dann auch die genannten Flags gesetzt. 

## thread.rs
