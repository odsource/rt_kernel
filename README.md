# rt_kernel
Realtime kernel based on the blog posts of Philipp Oppermann and extended by an EDF Scheduler approach.

Der Branch `master` enthält die aktuelle Version des Projekts!

![Kontextwechsel](https://github.com/odsource/rt_kernel/blob/master/QEMU_context_switch.png?raw=true)

## memory.rs
Für die Speicherverwaltung (Stack) wurde ein `struct StackFrame` erstellt, welches den Stackbereich jedes Threads darstellt. Der Bereich wird über zwei virtuelle Adressen (Start und Ende) abgebildet. Um solch einen Stackbereich zu erhalten muss die Funktion `get_stack_frame(mapper, frame_allocator)` aufgerufen werden. Dies passiert bei jeder Threaderstellung. 

Die Implementierung von `get_stack_frame()` ist ähnlich zur Heaperstellung des Blogs. Es wird ein allgemeiner Stack Start festgelegt (Adresse: 0x_8888_8888_0000). Bei jeder Threaderstellung wird nun ausgehend von der vorherigen Stackendadresse ein neuer Stackbereich erstellt. Um Probleme durch überlappende Adressen zu vermeiden wird immer ein Offset von einer Page hinzugefügt. Dies führt zwar zu Fragmentierung sichert aber dahingehend auch die Speicherbereiche einigermaßen ab. 

Um den Frame in den Speicher zu laden und ihn auch zugreifbar zu machen werden die Flags `PageTableFlags::PRESENT` und `PageTableFlags::WRITABLE` gesetzt. Bzw. wird der Bereich der virtuellen Adressen `stack_start` bis `stack_end` in Pages unterteilt und für jede Page ein Mapping auf einen zugehörigen Frame gemacht. Hierbei werden dann auch die genannten Flags gesetzt. 

## thread.rs
Die Datei `thread.rs` ist vom Grundprinzip gleich wie die im Blog definierten Tasks aufgebaut, jedoch auf den EDF Scheduler angepasst. Im Folgenden wird nur auf die Unterschiede eingegangen.

Jeder Thread hat wie zuvor erwähnt seinen eigenen Stackbereich. Dieser Bereich wird in dem Feld `stack_frame` gespeichert und der dazu gehörige Stackpointer in `stack_ptr`. Wenn nun ein neuer Thread erzeugt werden soll, wird als erstes der in `memory.rs` beschriebene Aufruf `get_stack_frame()` getätigt um einen allozierten Speicherbereich zu erhalten. Im nächsten Schritt wird ein Stackpointer auf das Ende des Stacks erzeugt und der Funktionspointer, sowie die benötigten Flags auf den Stack geschrieben (dazu später mehr in `context_switch.rs`). Der Rest ist hierbei selbsterklärend bzw. gleich wie in `task.rs`.

## context_switch.rs
In `context_switch.rs` finden die namensgebenden Kontextwechsel, sowie die Stackfüllung statt. Die Funktion `switch_context(new_stack_ptr)` sorgt für den Kontextwechsel, indem sie den neuen Stackpointer im Register `rsi` speichert, den alten Stackpointer zwischenspeichert (in `rax`) und den neuen Stackpointer in `rsp` schreibt. Im Anschluss wird der alte Stackpointer in das Register `rdi` geschrieben. Dies findet aufgrund des folgenden Funktionsaufrufs statt. Laut ​eines Cheatsheets für Assembler der Brown University werden die Register `rdi`​,​ `rsi`​, `rdx`​, `rcx`​, `r8`​, und `r9`, um Parameter an einen Funktionsaufruf weiterzugeben. Somit wird beim Aufruf `call old_stack_ptr` der alte Stackpointer an die Funktion übergeben. Wichtig zu erwähnen ist noch, das wir anfangs `pushfq` und am Schluss `popfq` aufrufen. Mit diesen Aufrufen werden die RFLAGS (Flags wie Zero, Sign, Interrupt, ...) in den Stack gepushed und vom Stack wieder gepopped. Dadurch wird sichergestellt, dass vor dem Kontextwechsel alle wichtigen Registerzustände im Stack gespeichert werden und nach dem Kontextwechsel wieder die richtigen vom Stack geholt werden.

In der Funktion `old_stack_ptr(old_ptr)` wird der alte Stackpointer gespeichert und der Stackpointer des entsprechenden Threads auf diesen gesetzt, da sonst einfach immer wieder die Funktion von vorne gestartet wird, auch wenn die Funktion gerade in der Mitte gestoppt wurde.

Zu guter letzt ist noch die Stack Implementierung für den Kontextwechsel, bzw. für die Threads zu finden. Wie schon zuvor gesagt, besteht das `struct Stack` nur aus einem Pointer auf eine virtuelle Adresse. Nachdem der Stack mit einer Stackendadresse initialisiert wurde zeigt der Pointer auf eben diese. Wird nun bei der Threaderzeugung die `write()` Funktion aufgerufen, wird der übergebene Wert auf den Stack geschrieben. Dazu wird zuerst die Größe über die `mem::size_of::<T>()` Methode aus dem `core` crate ermittelt. Um diesen Speicherbedarf wird der Stackpointer verringert. Verringert deshalb, da der Stack negativ wächst also von hohen zu niedrigen Adressen (laut Literatur ist dies nicht in jedem System so, aber in den meisten). Nun benötigen wir wie schon einige Male im Blog gezeigt einen Raw Pointer um die Methode `write()` zu nutzen, welche den übergebenen Wert auf den Stack bzw. in den Speicher schreibt. Dies passiert bei der Threaderzeugung zuerst für den Funktionspointer und im Anschluss für den Wert `0x200`. Der Wert `0x200` repräsentiert die RFLAGS. D.h. wenn der Kontextwechsel stattfindet werden durch das `popfq` die Interrupts wieder aktiviert. Die Reihenfolge ist hierbei auch wichtig da der Stackpointer zu Beginn auf diesen Wert zeigen muss. Sollte er auf den Funktionspointer zeigen, würde kein Timerinterrupt mehr stattfinden und somit kein Kontextwechsel mehr und der Thread würde sich nie beenden.

## scheduler.rs (mod.rs)
Nun zum EDF Scheduler an sich. Die Schedulerinstanz selbst wird, wie der `ALLOCATOR`, in einem `lazy_static!` Block ausgeführt und wird durch ein `.lock()` mutabel und zugreifbar. Ebenfalls wird sichergestellt, dass nur an einer Stelle gleichzeitig auf den Scheduler zugegriffen werden kann. Das `struct EDFScheduler` selbst hat folgende Felder:

- init: gibt an ob der Scheduler gestartet wurde
- tasks: eine `BTreeMap` über alle Threads, wobei die Deadline der Key ist
- active_task: die globale Deadline des aktuellen Thread
- old_task: die globale Deadline des vorherigen Thread

Bei der Erzeugung der Schedulerinstanz wird der Scheduler nur mit "leeren" Werten initialisiert bzw. eine `BTreeMap` erzeugt. Gefüllt wird der Scheduler erst, sobald Threads hinzugefügt werden. Dabei wird zuerst geschaut, ob der Thread aufgrund der momentanen CPU Auslastung überhaupt hinzugefügt werden kann. Dafür wird über alle Threads iteriert,
für jeden Thread Runtime/Period berechnet und das Ganze aufsummiert. Ist der Wert kleiner gleich 1 (inklusive potentiellem neuen Thread) kann er hinzugefügt werden. Dies ist zwar nur ein theoretischer Wert, welcher in der Praxis irreal ist, der Einfachheit halber aber so von uns implementiert wurde. Zu beachten ist hierbei noch das wir einen globalen Timer `GLOBAL_TIME` haben, welcher mit jedem Timerinterrupt hochgezählt wird. Dieser bildet die Basis für die Deadline Berechnung, da die Ankunftszeit durch ihn bestimmt wird.


Sind nun die ersten Threads beim Scheduler registriert wird die Methode `start()` aufgerufen. Dies führt dazu, dass der Scheduler den ersten Thread heraussucht, der laufen soll. Dazu wird die Methode `select_thread()` genutzt. In dieser wird aus dem `BTreeMap` die kleinste globale Deadline herausgesucht, `old_task` mit dem Wert des zuvor aktiven Threads gefüllt und `active_task` mit dem neuen Key überschrieben und der Thread zusammen mit seiner zugehörigen Deadline bis zum Aufruf durchgereicht. Im Fall von `start()` wird direkt ein Kontextwechsel zu dem ausgewählten Thread durchgeführt (siehe oben). 

Der Scheduler führt nun zu jedem Timerinterrupt seine `schedule()` Methode aus, um anhand der nächsten fälligen Deadline einen Thread auszuwählen. Ist ein Thread komplett durchgelaufen wird er der `BTreeMap` wieder hinzugefügt, allerdings mit neuer globaler Deadline ausgehend von der `GLOBAL_TIME` und zurückgesetzter Laufzeit. Im Anschluss wird wieder die Methode `select_thread()` ausgeführt. Es wurde auch noch eine Methode `yield_thread()` hinzugefügt, diese haben wir allerdings nicht mehr implementieren können.

## interrupt.rs
Wie schon zuvor genannt haben wir einen globalen Timer `GLOBAL_TIME` hinzugefügt, welcher bei jedem Timerinterrupt hochzählt. Zusätzlich wird hier auch jedes mal die Methode `schedule()` des EDF Schedulers aufgerufen und sollte ein Thread zurückgeliefert werden ein Kontextwechsel durchgeführt. Ein Fallstrick hierbei, war es den `notify_end_of_interrupt()` des `PICS` vor dem Kontextwechsel aufzurufen, da der Timerinterrupt sonst ausgeschaltet ist. Dies passiert, da wir nicht mehr aus dem Kontextwechsel zurückkommen und somit (sollte der `PICS` Aufruf danach kommen) kein Aufruf stattfinden kann.

## .travis.yml
Leider konnten wir Travis nicht erfolgreich implementieren, da das Target nicht gefunden werden konnte und online keine Hilfestellung dazu gefunden wurde. Die selbe Fehlermeldung lässt sich lokal replizieren, indem wir `#![no_std]` auskommentieren. Allerdings macht es keinen Sinn, dass es lokal auskommentiert zur gleichen Fehlermeldung wie bei Travis kommt, obwohl dort `#![no_std]` nicht auskommentiert ist.