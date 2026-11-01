import { useEffect, useRef } from "react";
import type {
  CSSProperties,
  DialogHTMLAttributes,
  InputHTMLAttributes,
  SelectHTMLAttributes,
} from "react";
import { select } from "../styles/ui";

// Matches the option height and list padding in app.css.
const OPTION_HEIGHT = 32;
const LIST_PADDING = 8;

// Choose the side the list opens on before it opens, so the CSS can square
// the corners where the button and the list meet.
function chooseDropDirection(element: HTMLSelectElement) {
  const rect = element.getBoundingClientRect();
  const needed = element.options.length * OPTION_HEIGHT + LIST_PADDING;
  const below = window.innerHeight - rect.bottom;
  element.dataset.drop = below < needed && rect.top > below ? "up" : "down";
}

export function Select({
  className,
  ...props
}: SelectHTMLAttributes<HTMLSelectElement>) {
  return (
    <select
      {...props}
      className={`${select} ${className ?? ""}`}
      onPointerDown={(event) => chooseDropDirection(event.currentTarget)}
      onKeyDown={(event) => chooseDropDirection(event.currentTarget)}
    />
  );
}

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

type SheetProps = Omit<DialogHTMLAttributes<HTMLDialogElement>, "onCancel"> & {
  onClose: () => void;
};

// A modal sheet. It opens on mount; the parent closes it by unmounting it.
export function Sheet({ onClose, className, ...props }: SheetProps) {
  const ref = useRef<HTMLDialogElement>(null);
  useEffect(() => {
    ref.current?.showModal();
  }, []);
  return (
    <dialog
      {...props}
      ref={ref}
      className={`m-auto max-h-[calc(100vh-48px)] w-[min(520px,calc(100vw-32px))] flex-col overflow-hidden rounded-[14px] border-0 bg-app p-0 text-ink shadow-control backdrop:bg-black/40 open:flex ${className ?? ""}`}
      onCancel={(event) => {
        event.preventDefault();
        onClose();
      }}
    />
  );
}
