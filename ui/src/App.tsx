import './App.css'

import { useState, useEffect } from 'react'

interface RustMessage {
	message_type: string;
	data: string;
}

declare global {
	interface Window {
		ipc?: {
			postMessage: (message: string) => void;
		};
	}
}

function App() {
	const [input, setInput] = useState('')
	const [messages, setMessages] = useState<string[]>([])

	useEffect(() => {
		// Listen for messages from Rust
		const handleRustMessage = (event: CustomEvent<RustMessage>) => {
			const message = event.detail;
			setMessages(prev => [...prev, `Received: ${message.data}`]);
		};

		// Add event listener
		window.addEventListener('rust-message', handleRustMessage as EventListener);

		// Cleanup
		return () => {
			window.removeEventListener('rust-message', handleRustMessage as EventListener);
		};
	}, []);

	const sendMessage = () => {
		if (!input.trim()) return;

		const message: RustMessage = {
			message_type: 'command',
			data: input
		};

		// Send message to Rust
		window.ipc?.postMessage(JSON.stringify(message));

		setMessages(prev => [...prev, `Sent: ${input}`]);
		setInput('');
	};

	return (
		<div className="p-4">
			<h1 className="text-2xl font-bold mb-4">My Application</h1>

			<div className="border p-4 mb-4 h-[400px] overflow-y-auto">
				{messages.map((msg, i) => (
					<div key={i} className="mb-2">{msg}</div>
				))}
			</div>

			<div className="flex gap-2">
				<input
					type="text"
					value={input}
					onChange={(e) => setInput(e.target.value)}
					onKeyPress={(e) => e.key === 'Enter' && sendMessage()}
					className="flex-1 px-2 py-1 border rounded"
					placeholder="Enter command..."
				/>
				<button
					onClick={sendMessage}
					className="px-4 py-1 bg-blue-500 text-white rounded"
				>
					Send
				</button>
			</div>
		</div>
	)
}

export default App
