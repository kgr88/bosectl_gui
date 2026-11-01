import type { CSSProperties, InputHTMLAttributes } from "react";

type SliderProps = Omit<InputHTMLAttributes<HTMLInputElement>, "type"> & {
  min: number;
  max: number;
  value: number;
  vertical?: boolean;
};

export function Slider({ vertical, className, ...props }: SliderProps) {
  const span = props.max - props.min;
  const fill = span > 0 ? ((props.value - props.min) / span) * 100 : 0;
  return (
    <input
      {...props}
      type="range"
      className={`${vertical ? "vertical" : ""} ${className ?? ""}`}
      style={{ "--fill": `${fill}%` } as CSSProperties}
    />
  );
}

type ToggleProps = Omit<InputHTMLAttributes<HTMLInputElement>, "type">;

export function Toggle({ className, ...props }: ToggleProps) {
  return (
    <span className={`relative inline-flex shrink-0 ${className ?? ""}`}>
      <input
        {...props}
        type="checkbox"
        role="switch"
        className="peer h-[31px] w-[51px] cursor-pointer appearance-none rounded-full bg-fill transition-colors checked:bg-success disabled:cursor-default disabled:opacity-40 forced-colors:appearance-auto"
      />
      <span
        aria-hidden="true"
        className="pointer-events-none absolute top-0.5 left-0.5 size-[27px] rounded-full bg-white shadow-control transition-transform peer-checked:translate-x-5 forced-colors:hidden"
      />
    </span>
  );
}
