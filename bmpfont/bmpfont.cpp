#include <stdio.h>
#include <QtWidgets/QApplication>
#include <QtWidgets/QMainWindow>
#include <QtWidgets/QVBoxLayout>
#include <QtWidgets/QHBoxLayout>
#include <QtWidgets/QPushButton>
#include <QtWidgets/QFontDialog>
#include <QtWidgets/QFileDialog>
#include <QtGui/QPainter>
#include <QtGui/QFont>
#include <string>

std::vector<std::string> chars = {
	" ", "!", "\"", "#", "$", "%", "&", "'", "(", ")", "*", "+", ",", "-", ".", "/",
	"0", "1", "2", "3", "4", "5", "6", "7", "8", "9", ":", ";", "<", "=", ">", "?",
	"@", "A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M", "N", "O",
	"P", "Q", "R", "S", "T", "U", "V", "W", "X", "Y", "Z", "[", "\\", "]", "^", "_",
	"`", "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o",
	"p", "q", "r", "s", "t", "u", "v", "w", "x", "y", "z", "{", "|", "}", "~", "ᴇ",
	"∞", "×", "÷", "±", "°", "∀", "∅", "∈", "∉", "∙", "∫", "≈", "≤", "≥", "⋂", "⋃",
	"←", "↑", "→", "↓", "↵", "⬏", "α", "β", "Γ", "γ", "Δ", "δ", "ϵ", "ϝ", "ζ", "η",
	"Θ", "θ", "ι", "κ", "Λ", "λ", "μ", "ν", "Ξ", "ξ", "Π", "π", "ρ", "Σ", "σ", "τ",
	"υ", "Φ", "ϕ", "χ", "Ψ", "ψ", "Ω", "ω", "…", "▪", "◂", "▴", "▸", "▾", "≠", "≷",
	"∡", "²", "³", "ˣ", "₂", "ℹ", "⟪", "⟫", "⦗", "⦘"};

class MainWidget : public QWidget
{
	QFont m_font;

public:
	MainWidget()
	{
		setFont(font());
	}

	QFont currentFont()
	{
		return m_font;
	}

	void setFont(const QFont &font)
	{
		m_font = font;
		m_font.setHintingPreference(QFont::PreferFullHinting);
		m_font.setStyleHint(QFont::AnyStyle, QFont::NoAntialias);
		update();
	}

protected:
	virtual void paintEvent(QPaintEvent *)
	{
		QPainter p(this);
		QString s;
		for (int y = 0; y < 6; y++)
		{
			for (int x = 0; x <= 32; x++)
			{
				if (((y * 32) + x) > chars.size())
					break;
				s += QString::fromStdString(chars[y * 32 + x]);
			}
			s += "\n";
		}
		p.setFont(m_font);
		p.drawText(rect(), s);
	}
};

class MainWindow : public QMainWindow
{
	MainWidget *m_widget;

public:
	MainWindow()
	{
		resize(800, 400);

		QWidget *container = new QWidget();
		QVBoxLayout *layout = new QVBoxLayout();
		m_widget = new MainWidget();
		layout->addWidget(m_widget, 1);
		QHBoxLayout *buttonLayout = new QHBoxLayout();
		QPushButton *fontButton = new QPushButton("Font...");
		buttonLayout->addWidget(fontButton);
		QPushButton *saveButton = new QPushButton("Save...");
		buttonLayout->addWidget(saveButton);
		buttonLayout->addStretch(1);
		layout->addLayout(buttonLayout);
		container->setLayout(layout);
		setCentralWidget(container);

		connect(fontButton, &QPushButton::pressed, this, &MainWindow::fontButton);
		connect(saveButton, &QPushButton::pressed, this, &MainWindow::saveButton);
	}

private:
	void fontButton()
	{
		bool ok;
		QFont font = QFontDialog::getFont(&ok, m_widget->currentFont());
		if (ok)
			m_widget->setFont(font);
	}

	void saveButton()
	{
		QString name = QFileDialog::getSaveFileName();
		if (name.isNull())
			return;
		std::string filename = name.toStdString();

		QFont font = m_widget->currentFont();
		QFontMetrics metrics(font);
		int charHeight = metrics.height();
		QImage image(100, 100, QImage::Format_ARGB32);

		FILE *fp = fopen(filename.c_str(), "w");
		fprintf(fp, "#[allow(dead_code)]\n");
		fprintf(fp, "pub const FONT: crate::screen::Font = crate::screen::Font {\n");
		fprintf(fp, "    height: %d,\n", charHeight);
		fprintf(fp, "    chars: &[\n");
		for (auto &ch : chars)
		{
			image.fill(QColor(255, 255, 255));
			QPainter p(&image);
			QRect r = metrics.boundingRect(QString::fromStdString(ch));
			r.setY(0);
			r.setHeight(charHeight);
			r.setX(0);
			int advance = metrics.horizontalAdvance(QString::fromStdString(ch));
			p.setFont(font);
			p.setPen(QColor(0, 0, 0));
			p.drawText(r, QString::fromStdString(ch));
			fprintf(fp, "        &[");
			for (int y = 0; y < charHeight; y++)
			{
				for (int xByte = 0; xByte < r.x() + r.width(); xByte += 8)
				{
					unsigned int value = 0;
					int left = r.x() + r.width() - xByte;
					if (left > 8)
						left = 8;
					for (int x = 0; x < left; x++)
					{
						if (qBlue(image.pixel(xByte + x, y)) < 128)
							value |= 1 << ((left - 1) - x);
					}
					fprintf(fp, "0x%x,", value);
				}
			}
			fprintf(fp, "],\n");
		}
		fprintf(fp, "    ],\n");
		fprintf(fp, "    width: &[\n        ");
		for (size_t i = 0; i < chars.size(); i++)
		{
			if ((i > 0) && ((i % 0x20) == 0))
				fprintf(fp, "\n        ");
			QRect r = metrics.boundingRect(QString::fromStdString(chars[i]));
			r.setX(0);
			fprintf(fp, "%d,", r.x() + r.width());
		}
		fprintf(fp, "\n    ],\n");
		fprintf(fp, "    advance: &[\n        ");
		for (size_t i = 0; i < chars.size(); i++)
		{
			if ((i > 0) && ((i % 0x20) == 0))
				fprintf(fp, "\n        ");
			fprintf(fp, "%d,", metrics.horizontalAdvance(QString::fromStdString(chars[i])));
		}
		fprintf(fp, "\n    ],\n");
		fprintf(fp, "};\n");
		fclose(fp);
	}
};

int main(int argc, char *argv[])
{
	QCoreApplication::setAttribute(Qt::AA_EnableHighDpiScaling);
	QCoreApplication::setAttribute(Qt::AA_UseHighDpiPixmaps);

	QApplication app(argc, argv);

	MainWindow window;
	window.show();
	return app.exec();
}
