import clsx from 'clsx';

export function HSep() {
  return <div className="hsep bg-zinc-800" />;
}

type SepProps = {
  className?: string;
};

export function VSep({ className = '' }: SepProps) {
  return (
    <div
      className={clsx('vsep bg-zinc-800', {
        [className]: className.length > 0,
      })}
    />
  );
}
