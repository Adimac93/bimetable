import dayjs from "./dayjs";

export interface BaseCalendarEvent {
    name: string;
    description?: string;
    startTime?: dayjs.Dayjs;
    endTime?: dayjs.Dayjs;
}

export class CalendarEvent implements BaseCalendarEvent {
    id: string;
    name: string;
    description?: string;
    startTime?: dayjs.Dayjs;
    endTime?: dayjs.Dayjs;

    constructor(id: string, name: string, description?: string, start?: dayjs.Dayjs, end?: dayjs.Dayjs) {
        if (!(start && end)) {
            throw new Error("Invalid event (no start and end)");
        }

        this.id = id;
        this.name = name;
        this.description = description;
        this.startTime = start;
        this.endTime = end;
    }

    // Returns the day the event starts on.
    get day() {
        const day = (this.startTime ?? this.endTime)?.startOf("day");
        if (!day) {
            throw new Error("Invalid event (no start and end)");
        }
        return day;
    }

    clone() {
        return new CalendarEvent(this.id, this.name, this.description, this.startTime, this.endTime);
    }
}
