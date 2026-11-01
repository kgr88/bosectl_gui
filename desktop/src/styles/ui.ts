export const button =
  "min-h-[34px] cursor-pointer rounded-full bg-accent px-[18px] py-1.5 font-medium text-white transition-opacity hover:opacity-90 active:opacity-75 disabled:cursor-default disabled:opacity-40";
export const secondaryButton =
  "min-h-[34px] cursor-pointer rounded-full bg-fill px-[18px] py-1.5 font-medium text-accent transition-opacity hover:opacity-80 disabled:cursor-default disabled:opacity-40";
export const textButton =
  "cursor-pointer border-0 bg-transparent p-0 text-accent hover:opacity-75 disabled:cursor-default disabled:opacity-40";
export const dangerTextButton = `${textButton} text-danger`;
export const input =
  "min-h-[34px] rounded-[10px] border-0 bg-fill px-3 py-1.5 text-ink placeholder:text-muted";
export const select = `${input} cursor-pointer pr-2`;
export const section = "mt-7";
export const sectionHeader =
  "mb-1.5 px-4 text-[13px] font-normal tracking-[0.02em] text-muted uppercase";
export const sectionFooter = "mt-1.5 px-4 text-[13px] leading-5 text-muted";
export const group =
  "overflow-hidden rounded-[14px] bg-surface shadow-card forced-colors:border forced-colors:border-[CanvasText]";
export const groupList = `${group} m-0 list-none divide-y divide-separator p-0`;
export const row =
  "flex min-h-[48px] items-center justify-between gap-3 px-4 py-2.5";
export const rowDetail = "mt-0.5 block text-[13px] text-muted";
export const pageGrid =
  "grid grid-cols-[repeat(auto-fit,minmax(320px,1fr))] gap-x-6 max-[600px]:grid-cols-1";
