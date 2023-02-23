<template>
    <div>
        <ul>
            <li v-for="event in events">{{ event.name }}: {{ event.day.format('DD MM YYYY') }}</li>
        </ul>
    </div>
</template>

<script setup lang="ts">
import dayjs from '@/utils/dayjs';
import { CalendarEvent } from '~~/utils/CalendarEvent';

const events = [
    new CalendarEvent(
        'Bibruspotkanie',
        dayjs(1675004400000), // 2023-01-29 15:00:00
        dayjs(1675008000000) // 2023-01-29 16:00:00
    ),
    new CalendarEvent('Coś na pewno', dayjs(1675072800000), dayjs(1675101600000)),
    new CalendarEvent('Podróż w czasie', dayjs(1673082000000), dayjs(1673082000001)),
];
</script>
