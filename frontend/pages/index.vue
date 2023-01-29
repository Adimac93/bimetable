<template>
    <div>
        <h1>Bimetable</h1>
        <div class="wrapper">
            <div>
                <Calendar :events="events" @select="selectDay"/>
            </div>
            <div class="side-view">
                <template v-if="selected">
                    <h2>{{ selected.date.format("DD MMM YYYY") }}</h2>
                    <EventCard v-for="event in selected.events" :event="event"/>
                </template>
                <template v-else>
                    select a day with events please
                </template>
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import dayjs from "@/utils/dayjs";
import type { CalendarEvent } from "@/utils/CalendarEvent";

const events = [
    {
        name: "Bibruspotkanie",
        startTime: 1675004400000, // 2023-01-29 15:00:00
        endTime: 1675008000000, // 2023-01-29 16:00:00
    },
    {
        name: "Coś na pewno",
        startTime: 1675072800000,
        endTime: 1675101600000,
    },
    {
        name: "Podróż w czasie",
        startTime: 1673082000000,
        endTime: 1673082000001,
    }
];

const selected = ref<{ date: dayjs.Dayjs, events: CalendarEvent[] } | null>(null);

function selectDay(data: { date: dayjs.Dayjs, events: CalendarEvent[] } | null) {
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