<template>
    <Week :events="events" class="week" />
</template>

<script setup lang="ts">
import dayjs from "@/utils/dayjs";
import { CalendarEvent } from "@/utils/CalendarEvent";

const events: CalendarEvent[] = [];

let start = dayjs().startOf("week");

// just a mock
for (let day = 0; day < 7; day++) {
    start = start.startOf("day").add(8, "hours");
    for (let i = 0; i < 9; i++) {
        let end = start.add(45, "minutes");
        events.push(new CalendarEvent("Event", start, end));
        start = end.add(10, "minutes");
    }
    start = start.add(1, "day");
}
</script>

<style scoped>
/* Without a fixed height the pinch/zoom functionality breaks */
.week {
    height: 80svh;
}
</style>
