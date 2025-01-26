import { useState } from 'react';
import BadgeSelector from './ui/BadgeSelector';
import demoLight from '../assets/demo_light.webp';

interface IDemo {
    type: 'video' | 'image';
    title: string;
    src: any; // Astro's image type is ?
    alt?: string;
    disabled?: boolean;
}

export default function ProductDemo() {
    const demos: IDemo[] = [
        {
            alt: 'product demo - recording',
            title: 'Record Anything',
            src: demoLight.src || demoLight,
            type: 'image'
        },
        {
            alt: 'product demo - summary',
            title: 'Summarize Content',
            src: demoLight.src || demoLight,
            type: 'image'
        },
        {
            alt: 'product demo - search',
            title: 'Search Coming Soon!',
            src: demoLight.src || demoLight,
            type: 'image',
            disabled: true
        },
    ];

    const [selectedDemo, setSelectedDemo] = useState(demos[0]);
    const options = demos.map(demo => ({
        value: demo.title,
        disabled: demo.disabled
    }));

    return (
        <div className="flex flex-col items-center">
            <div className="pb-4">
                <BadgeSelector
                    options={options}
                    selected={selectedDemo.title}
                    setSelected={(title) => {
                        console.log("Setting Selected")
                        const demo = demos.find(d => d.title === title);
                        if (demo) setSelectedDemo(demo);
                    }}
                />
            </div>
            <div className="w-full bg-gray-100 rounded-lg p-1 bg-orange-verm mx-auto">
                <div className="bg-white rounded-lg shadow-md overflow-hidden">
                    <div className="aspect-w-16 aspect-h-9">
                        {selectedDemo.type === 'video' ? (
                            <video className="w-full h-full object-cover" controls autoPlay muted loop>
                                <source src={selectedDemo.src} type="video/mp4" />
                            </video>
                        ) : (
                            <img
                                src={selectedDemo.src}
                                alt={selectedDemo.alt || 'Product demo'}
                                className="w-full h-full object-cover"
                            />
                        )}
                    </div>
                </div>
            </div>
        </div>
    );
}
