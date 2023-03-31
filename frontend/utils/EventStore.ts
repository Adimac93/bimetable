import { useAPI } from "~~/composables/useAPI";
import { BaseCalendarEvent, CalendarEvent, StoredCalendarEvent } from "./CalendarEvent";
import dayjs from "./dayjs";

// TODO: implement overrides
export interface EventEntry {
    eventID: string;
    startsAt: dayjs.Dayjs;
    endsAt: dayjs.Dayjs;
}

export type ProcessedEvent = CalendarEvent & {
    raw: StoredCalendarEvent;
    entry: EventEntry;
};

class EventIteratorContext {
    private store: EventStore;
    private startAt: number;
    private endAt: number;

    constructor(store: EventStore) {
        this.store = store;
        this.startAt = 0;
        this.endAt = -1;
    }

    // Sets the starting point. Can be a date or event index. Inclusive. Default 0.
    from(startPoint: dayjs.Dayjs | number) {
        if (typeof startPoint != "number") {
            this.startAt = this.store.findIndexAfter(startPoint) ?? this.store.entries.length;
        } else {
            this.startAt = startPoint;
        }
        return this;
    }

    // Sets the ending point. Can be a date or event index. Exclusive. Default -1 (end of the list)
    to(endPoint: dayjs.Dayjs | number) {
        if (typeof endPoint != "number") {
            const val = this.store.findIndexBefore(endPoint);
            if (val == null) {
                this.endAt = 0;
            } else {
                this.endAt = val + 1;
            }
        } else {
            this.endAt = endPoint;
        }
        return this;
    }

    // Sets the ending point by adding a number to the starting point.
    next(count: number) {
        this.endAt = this.startAt + count;
        return this;
    }

    collect(): ProcessedEvent[] {
        const result = [];
        for (const event of this) {
            result.push(event);
        }
        return result;
    }

    *[Symbol.iterator]() {
        let startIndex = this.startAt;

        let endIndex: number;
        if (this.endAt < 0) {
            endIndex = this.store.entries.length;
        } else {
            endIndex = this.endAt;
        }

        if (startIndex > endIndex) {
            throw new RangeError("Invalid search bounds");
        }

        for (let i = startIndex; i < endIndex; i++) {
            const entry = this.store.entries[i];
            const original = this.store.data.get(entry.eventID)!;
            const incomplete: Partial<ProcessedEvent> = new CalendarEvent(
                original.id,
                original.name,
                original.description,
                dayjs(), // these are placeholders
                dayjs() // and are replaced later in the code
            );
            incomplete.raw = original;
            incomplete.entry = entry;

            const event = incomplete as ProcessedEvent;
            incomplete.startTime = entry.startsAt;
            incomplete.endTime = entry.endsAt;
            // TODO: implement overrides
            yield event;
        }
    }
}

export class EventStore {
    // Chronological list of entries
    entries: EventEntry[];
    data: Map<string, StoredCalendarEvent>;
    start: dayjs.Dayjs;
    end: dayjs.Dayjs;

    constructor(start: dayjs.Dayjs, end: dayjs.Dayjs) {
        this.entries = [];
        this.data = new Map();
        this.start = start;
        this.end = end;
    }

    findIndexBefore(timestamp: dayjs.Dayjs): number | null {
        // TODO: test
        let start = 0;
        let end = this.entries.length - 1;
        while (start <= end) {
            // get the middle
            const mid = Math.floor((start + end) / 2);

            const el = this.entries[mid];
            const next = this.entries[mid + 1];
            if (!next) {
                // the last element may be appropriate
                return el.startsAt.unix() < timestamp.unix() ? mid : null;
            }

            if (el.startsAt.unix() < timestamp.unix() && next.startsAt.unix() >= timestamp.unix()) {
                return mid;
            } else if (el.startsAt.unix() < timestamp.unix()) {
                // both less
                start = mid + 1;
            } else {
                // both more (next is guaranteed to be after el)
                end = mid - 1;
            }
        }
        return null;
    }

    // This is really "not before" - it will find a value exactly at the timestamp if one exists
    findIndexAfter(timestamp: dayjs.Dayjs): number | null {
        if (this.entries.length > 0 && this.entries[0].startsAt.unix() >= timestamp.unix()) {
            return 0;
        }

        const result = this.findIndexBefore(timestamp);
        if (result == null) return null;
        if (result + 1 == this.entries.length) {
            return null;
        }
        return result + 1;
    }

    iter() {
        return new EventIteratorContext(this);
    }

    // Bounds of date range that is loaded, specifically all loaded events are guaranteed to be
    // contained in [start, end)
    get loadedBounds() {
        return {
            start: this.entries[0].startsAt,
            end: this.entries[this.entries.length - 1].endsAt.add(1, "second"),
        };
    }

    // Load enough events so everything from newStart to the existing end is loaded.
    async extendFrom(newStart: dayjs.Dayjs) {
        // TODO: request events in [newStart, loadedBounds.start)
    }

    // Load enough events so everything from the existing start to newEnd is loaded.
    async extendTo(newEnd: dayjs.Dayjs) {
        // TODO: request events in [loadedBounds.end, newEnd)
    }

    async createEvent(event: BaseCalendarEvent) {
        // TODO
    }

    async deleteEvent(eventID: string) {
        // TODO: actually query the API
        if (!this.data.delete(eventID)) {
            throw new Error(`No event with ID ${eventID}`);
        }
        // remove them from the entries list
        this.entries = this.entries.filter((el) => el.eventID != eventID);
    }
}

export interface EventStoreAPIData {
    entries: { eventID: string; startTime: string; endTime: string }[];
    data: Record<string, { name: string; description: string; startTime: string; endTime: string }>;
}

// You should probably use `fetchEventStore` instead`
export function makeEventStore(data: EventStoreAPIData) {
    const store = reactive(
        new EventStore(
            dayjs(data.entries[0].startTime),
            dayjs(data.entries[data.entries.length - 1].startTime).add(1, "second")
        )
    );
    store.entries = data.entries.map((entry) => ({
        eventID: entry.eventID,
        startsAt: dayjs(entry.startTime),
        endsAt: dayjs(entry.endTime),
    }));

    for (const [uuid, value] of Object.entries(data.data)) {
        store.data.set(uuid, {
            id: uuid,
            name: value.name,
            description: value.description,
            repetitionStart: dayjs(value.startTime),
            repetitionEnd: dayjs(value.endTime),
        });
    }
    return store;
}

interface APIEventsGetResult {
    entries: {
        starts_at: string;
        ends_at: string;
        event_id: string;
        recurrence_override?: {
            created_at: string;
            deleted_at: string;
            name: string;
            description: string;
        };
    }[];
    events: Record<
        string,
        {
            can_edit: boolean;
            is_owned: boolean;
            payload: {
                name: string;
                description?: string;
            };
        }
    >;
}

// Fetch the event data in an event range [start, end) from the API.
export async function fetchEventStore(start: dayjs.Dayjs, end: dayjs.Dayjs) {
    if (start > end) {
        throw new RangeError("Start of date range must be before end");
    }
    // TODO: test this (this will require putting some events in the database)
    const { data, pending, refresh, error } = await useAPI<APIEventsGetResult>("/api/events", {
        params: {
            starts_at: start.toISOString(),
            ends_at: end.toISOString(),
            filter: "all",
        },
        default: () => ({
            entries: [],
            events: {},
        }),
    });
    console.log(data.value);

    const store = ref<EventStore | null>(null);
    // put it in a store
    function putItInTheStore() {
        if (!data.value) {
            store.value = null;
            return;
        }

        if (!store.value) {
            store.value = new EventStore(start, end);
        }

        store.value!.entries = data.value.entries.map((entry) => ({
            eventID: entry.event_id,
            startsAt: dayjs(entry.starts_at),
            endsAt: dayjs(entry.ends_at),
        }));

        store.value!.data = new Map();
        for (const [id, apiEvent] of Object.entries(data.value.events)) {
            const ev = {
                id,
                name: apiEvent.payload.name,
                description: apiEvent.payload.description,
                // TODO: get this from the API when that is implemented
                repetitionStart: dayjs(new Date(NaN)),
                repetitionEnd: dayjs(new Date(NaN)),
            };
            store.value!.data.set(id, ev);
        }
    }

    putItInTheStore();
    watch(data, () => putItInTheStore());

    return { store, pending, refresh, error };
}
