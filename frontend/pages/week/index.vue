<template>
    <!-- TODO: iterate over only the week we're supposed to -->
    <Week :events="eventStore.iter().collect()" class="week" />
</template>

<script setup lang="ts">
import dayjs from "@/utils/dayjs";

const events = {
    entries: <{ eventID: string; startTime: string; endTime: string }[]>[],
    data: {
        event: {
            name: "Event",
            description: "Mock event",
            startTime: "",
            endTime: "",
        },
    },
};

let start = dayjs().startOf("week");

// just a mock
for (let day = 0; day < 7; day++) {
    start = start.startOf("day").add(8, "hours");
    for (let i = 0; i < 9; i++) {
        let end = start.add(45, "minutes");
        events.entries.push({
            eventID: "event",
            startTime: start.toISOString(),
            endTime: end.toISOString(),
        });
        start = end.add(10, "minutes");
    }
    start = start.add(1, "day");
}

events.data.event.startTime = events.entries[0].startTime;
events.data.event.endTime = events.entries[0].endTime;

const eventStore = makeEventStore(events);
</script>

<style scoped>
/* Without a fixed height the pinch/zoom functionality breaks */
.week {
    height: 80svh;
}
</style>
