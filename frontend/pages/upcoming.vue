<template>
    <div>
        <ClientOnly>
            <ul>
                <li v-for="event in events.iter()">
                    {{ event.name }}: {{ event.day.format("YYYY-MM-DD") }} ({{ event.startTime?.format("HH:mm") }} -
                    {{ event.endTime?.format("HH:mm") }})
                </li>
            </ul>
        </ClientOnly>
    </div>
</template>

<script setup lang="ts">
import { makeEventStore } from "@/utils/EventStore";

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
</script>
