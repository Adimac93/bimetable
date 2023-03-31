<template>
    <div>
        <h1>Bimetable playground</h1>
        <div class="wrapper">
            <div>
                <Calendar :events="events" @select="selectDay" />
            </div>
            <div class="side-view">
                <template v-if="selected">
                    <h2>{{ selected.date.format("DD MMM YYYY") }}</h2>
                    <EventCard v-for="event in selected.events" :event="event" />
                </template>
                <template v-else> select a day with events please </template>
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import dayjs from "@/utils/dayjs";
import type { CalendarEvent } from "@/utils/CalendarEvent";

const events = makeEventStore({
    entries: [
        {
            eventID: "a",
            startTime: "2023-03-05T16:00:00Z",
            endTime: "2023-03-05T17:00:00Z",
        },
        {
            eventID: "b",
            startTime: "2023-03-06T13:00:00Z",
            endTime: "2023-03-06T14:00:00Z",
        },
        {
            eventID: "a",
            startTime: "2023-03-07T16:00:00Z",
            endTime: "2023-03-07T17:00:00Z",
        },
        {
            eventID: "c",
            startTime: "2023-03-09T08:00:00Z",
            endTime: "2023-03-09T10:00:00Z",
        },
        {
            eventID: "b",
            startTime: "2023-03-13T13:00:00Z",
            endTime: "2023-03-13T14:00:00Z",
        },
    ],
    data: {
        a: {
            name: "A",
            description: "Zdarzenie A",
            startTime: "2023-03-05T16:00:00Z",
            endTime: "2023-03-05T17:00:00Z",
        },
        b: {
            name: "B",
            description: "Zdarzenie B",
            startTime: "2023-03-06T13:00:00Z",
            endTime: "2023-03-06T14:00:00Z",
        },
        c: {
            name: "C",
            description: "Zdarzenie C (nie powtarza się)",
            startTime: "2023-03-09T08:00:00Z",
            endTime: "2023-03-09T10:00:00Z",
        },
    },
});

const selected = ref<{ date: dayjs.Dayjs; events: CalendarEvent[] } | null>(null);

function selectDay(data: { date: dayjs.Dayjs; events: CalendarEvent[] } | null) {
    selected.value = data;
}
</script>

<style scoped lang="scss">
.wrapper {
    display: flex;
    flex-flow: row nowrap;
    gap: 12px;
}
</style>
