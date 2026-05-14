import type { ServiceView } from "@/lib/types";
import { ServiceCard } from "./ServiceCard";

type Props = {
  title: string;
  services: ServiceView[];
};

export function ServiceSection({ title, services }: Props) {
  if (services.length === 0) return null;

  return (
    <section className="flex flex-col gap-1">
      <div className="flex items-center justify-between px-1 pt-2 pb-1">
        <h2 className="text-[10px] font-semibold uppercase tracking-wider text-muted-foreground">
          {title}
        </h2>
        <span className="text-[10px] tabular-nums text-muted-foreground">
          {services.length}
        </span>
      </div>
      <ul className="flex flex-col gap-1">
        {services.map((s) => (
          <li key={s.id}>
            <ServiceCard service={s} />
          </li>
        ))}
      </ul>
    </section>
  );
}
