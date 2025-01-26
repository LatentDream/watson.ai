interface IOption {
  value: string;
  disabled?: boolean;
}

interface BadgeSelectorProps {
  options: IOption[];
  selected: string;
  setSelected: (value: string) => void;
  className?: string;
}

export default function BadgeSelector({ options, selected, setSelected, className }: BadgeSelectorProps) {
  return (
    <div className={`inline-flex space-x-2 p-1 bg-gray-100/50 rounded-full ${className || ''}`}>
      {options.map((option) => {
        const isSelected = selected === option.value;
        let cn = "cursor-pointer rounded-full text-black text-sm py-1 px-3";
        if (isSelected) {
          cn += ' bg-white-angel hover:bg-orange-verm hover:text-white-angel';
        } else if (option.disabled) {
          cn += ' bg-transparent cursor-not-allowed text-gray-500';
        } else {
          cn += ' bg-transparent hover:bg-gray-200';
        }

        return (
          <button
            key={option.value}
            className={cn}
            disabled={option.disabled}
            onClick={() => {
              console.log("Clicked")
              if (!option.disabled) {
                setSelected(option.value);
              }
            }}
          >
            {option.value}
          </button>
        );
      })}
    </div>
  );
}
